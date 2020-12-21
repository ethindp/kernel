// SPDX-License-Identifier: MPL-2.0
use bit_field::BitField;
use bootloader::bootinfo::*;
use core::ops::Range;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use log::*;
use minivec::MiniVec;
use rand_core::{RngCore, SeedableRng};
use rand_hc::Hc128Rng;
use spin::{Mutex, RwLock};
use x86::random;
use x86_64::{
    registers::control::*,
    structures::paging::mapper::MapToError,
    structures::paging::page::PageRangeInclusive,
    structures::paging::OffsetPageTable,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

lazy_static! {
/// The page table mapper (PTM) used by the kernel global memory allocator.
static ref MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
/// The global frame allocator (GFA); works in conjunction with the PTM.
static ref FRAME_ALLOCATOR: Mutex<Option<GlobalFrameAllocator>> = Mutex::new(None);
static ref MMAP: RwLock<MiniVec<MemoryRegion>> = RwLock::new(MiniVec::new());
static ref ADDRRNG: Mutex<Hc128Rng> = Mutex::new({
let mut seed = [0u8; 32];
unsafe {
random::rdseed_slice(&mut seed);
}
Hc128Rng::from_seed(seed)
});
}

static MUSE: AtomicU64 = AtomicU64::new(0);
static SMUSE: AtomicU64 = AtomicU64::new(0);
static STOTAL: AtomicU64 = AtomicU64::new(0);
static FPOS: AtomicU64 = AtomicU64::new(0);

/// Initializes a memory heap for the global memory allocator. Requires a PMO to start with.
unsafe fn init_mapper(physical_memory_offset: u64) -> OffsetPageTable<'static> {
    // Get active L4 table
    trace!(
        "Retrieving active L4 table with memoffset {:X}",
        physical_memory_offset
    );
    let (level_4_table, _) = get_active_l4_table(physical_memory_offset);
    // initialize the mapper
    OffsetPageTable::new(level_4_table, VirtAddr::new(physical_memory_offset))
}

/// Allocates a paged heap.
fn allocate_paged_heap(
    start: u64,
    size: u64,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    // Construct a page range
    let page_range = {
        // Calculate start and end
        let heap_start = VirtAddr::new(start);
        let heap_end = heap_start + size - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    // Allocate appropriate page frames
    page_range.for_each(|page| {
        let frame = match frame_allocator.allocate_frame() {
            Some(f) => f,
            None => panic!("Can't allocate frame!"),
        };
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let frame2 = frame;
        unsafe {
            match mapper.map_to(page, frame, flags, frame_allocator) {
                Ok(f) => {
                    f.flush();
                    MUSE.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => panic!(
                    "Cannot allocate frame range {:X}h-{:X}h: {:?}",
                    frame2.start_address().as_u64(),
                    frame2.start_address().as_u64() + frame2.size(),
                    e
                ),
            }
        }
    });
    SMUSE.fetch_add(size, Ordering::Relaxed);
}

/// Allocates a paged heap with the specified permissions.
/// Possible permissions are:
/// * Writable (W): controls whether writes to the mapped frames are allowed. If this bit is
/// unset in a level 1 page table entry, the mapped frame is read-only.
///     If this bit is unset in a higher level page table entry the complete range of mapped
/// pages is read-only.
/// * User accessible (UA): controls whether accesses from userspace (i.e. ring 3) are
/// permitted.
/// * Write-through (WT): if this bit is set, a "write-through" policy is used for the cache,
/// else a "write-back" policy is used.
/// * No cache (NC): Disables caching for this memory page.
/// * Huge page (HP): specifies that the entry maps a huge frame instead of a page table.
/// Only allowed in P2 or P3 tables.
/// * Global (G): indicates that the mapping is present in all address spaces, so it isn't
/// flushed from the TLB on an address space switch.
/// * bits 9, 10, 11, and 52-62: available to the OS, can be used to store additional data,
/// e.g. custom flags.
/// * No execute (NX): forbid code execution from the mapped frames. Can be only used when
/// the no-execute page protection feature is enabled in the EFER register.
fn allocate_paged_heap_with_perms(
    start: u64,
    size: u64,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    permissions: PageTableFlags,
) {
    let page_range = {
        let heap_start = VirtAddr::new(start);
        let heap_end = heap_start + size - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    page_range.for_each(|page| {
        let frame = match frame_allocator.allocate_frame() {
            Some(f) => f,
            None => panic!("Can't allocate frame!"),
        };
        let frame2 = frame;
        unsafe {
            match mapper.map_to(page, frame, permissions, frame_allocator) {
                Ok(f) => {
                    f.flush();
                    MUSE.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => panic!(
                    "Cannot allocate frame range {:X}h-{:X}h: {:?}",
                    frame2.start_address().as_u64(),
                    frame2.start_address().as_u64() + frame2.size(),
                    e
                ),
            }
        }
    });
    SMUSE.fetch_add(size, Ordering::Relaxed);
}

unsafe fn get_active_l4_table(physical_memory_offset: u64) -> (&'static mut PageTable, Cr3Flags) {
    let (table_frame, flags) = Cr3::read();
    let phys = table_frame.start_address();
    let virt = VirtAddr::new(phys.as_u64() + physical_memory_offset);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    (&mut *page_table_ptr, flags)
}

#[derive(Debug, Copy, Clone)]
pub struct GlobalFrameAllocator {
    memory_map: &'static MemoryMap,
}

impl GlobalFrameAllocator {
    pub fn init(memory_map: &'static MemoryMap) -> Self {
        GlobalFrameAllocator { memory_map }
    }
}

unsafe impl FrameAllocator<Size4KiB> for GlobalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        FPOS.fetch_add(1, Ordering::SeqCst);
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
            .nth(FPOS.load(Ordering::SeqCst) as usize)
    }
}

pub fn init(
    physical_memory_offset: u64,
    memory_map: &'static MemoryMap,
    start_addr: u64,
    size: u64,
) {
    let mut mapper = MAPPER.lock();
    let mut allocator = FRAME_ALLOCATOR.lock();
    *mapper = Some(unsafe { init_mapper(physical_memory_offset) });
    *allocator = Some(GlobalFrameAllocator::init(memory_map));
    let end_addr = start_addr + size;
    match (mapper.as_mut(), allocator.as_mut()) {
        (Some(m), Some(a)) => allocate_paged_heap(start_addr, end_addr - start_addr, m, a),
        _ => panic!("Cannot acquire mapper or frame allocator lock!"),
    }
}

pub fn allocate_heap(start: u64, size: u64) {
    let mut mapper = MAPPER.lock();
    let mut allocator = FRAME_ALLOCATOR.lock();
    match (mapper.as_mut(), allocator.as_mut()) {
        (Some(m), Some(a)) => allocate_paged_heap(start, size, m, a),
        _ => panic!("Cannot acquire mapper or frame allocator lock!"),
    }
}

pub fn allocate_heap_with_perms(start: u64, size: u64, perms: PageTableFlags) {
    let mut mapper = MAPPER.lock();
    let mut allocator = FRAME_ALLOCATOR.lock();
    match (mapper.as_mut(), allocator.as_mut()) {
        (Some(m), Some(a)) => allocate_paged_heap_with_perms(start, size, m, a, perms),
        _ => panic!("Cannot acquire mapper or frame allocator lock!"),
    }
}

pub fn allocate_page_range(start: u64, end: u64) {
    let mut mapper = MAPPER.lock();
    let mut allocator = FRAME_ALLOCATOR.lock();
    match (mapper.as_mut(), allocator.as_mut()) {
        (Some(m), Some(a)) => {
            let page_range = {
                let start = VirtAddr::new(start);
                let end = VirtAddr::new(end);
                let start_page = Page::containing_address(start);
                let end_page = Page::containing_address(end);
                Page::range_inclusive(start_page, end_page)
            };
            page_range.for_each(|page| {
                let frame = match a.allocate_frame() {
                    Some(f) => f,
                    None => panic!("Can't allocate frame!"),
                };
                let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
                unsafe {
                    match m.map_to(page, frame, flags, a) {
                        Ok(r) => {
                            r.flush();
                            MUSE.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(e) => match e {
                            MapToError::PageAlreadyMapped(_) => (),
                            MapToError::FrameAllocationFailed => panic!(
                                "Cannot map frame at addr {:X} of size {}: no more frames",
                                frame.clone().start_address(),
                                frame.size()
                            ),
                            MapToError::ParentEntryHugePage => (),
                        },
                    }
                }
            });
        }
        _ => panic!("Memory allocator or frame allocator are not set"),
    }
    SMUSE.fetch_add(end - start, Ordering::Relaxed);
}

pub fn allocate_page_range_with_perms(start: u64, end: u64, permissions: PageTableFlags) {
    let mut mapper = MAPPER.lock();
    let mut allocator = FRAME_ALLOCATOR.lock();
    match (mapper.as_mut(), allocator.as_mut()) {
        (Some(m), Some(a)) => {
            let page_range = {
                let start = VirtAddr::new(start);
                let end = VirtAddr::new(end);
                let start_page = Page::containing_address(start);
                let end_page = Page::containing_address(end);
                Page::range_inclusive(start_page, end_page)
            };
            page_range.for_each(|page| {
                let frame = match a.allocate_frame() {
                    Some(f) => f,
                    None => panic!("Can't allocate frame!"),
                };
                unsafe {
                    match m.map_to(page, frame, permissions, a) {
                        Ok(r) => {
                            r.flush();
                            MUSE.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(e) => match e {
                            MapToError::PageAlreadyMapped(_) => (),
                            MapToError::FrameAllocationFailed => panic!(
                                "Cannot map frame at addr {:X} of size {}: no more frames",
                                frame.clone().start_address(),
                                frame.size()
                            ),
                            MapToError::ParentEntryHugePage => (),
                        },
                    }
                }
            });
        }
        _ => panic!("Memory allocator or frame allocator are not set"),
    }
    SMUSE.fetch_add(end - start, Ordering::Relaxed);
}

pub fn allocate_phys_range(start: u64, end: u64, force: bool) -> bool {
    let m = MMAP.read();
    let cnt = m
        .iter()
        .filter(|r| {
            r.region_type == MemoryRegionType::Usable
                && (r.start..r.end).contains(&start)
                && (r.start..r.end).contains(&end)
        })
        .count();
    if cnt > 0 || force {
        debug!(
            "Allocating memaddr {:X}h-{:X}h ({} bytes), flags: {:X}h",
            start,
            end,
            end - start,
            (PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE).bits()
        );
        let mut mapper = MAPPER.lock();
        let mut allocator = FRAME_ALLOCATOR.lock();
        match (mapper.as_mut(), allocator.as_mut()) {
            (Some(m), Some(a)) => {
                let frame_range = {
                    let start = PhysAddr::new(start);
                    let end = PhysAddr::new(end);
                    let start_frame = PhysFrame::<Size4KiB>::containing_address(start);
                    let end_frame = PhysFrame::<Size4KiB>::containing_address(end);
                    PhysFrame::range_inclusive(start_frame, end_frame)
                };
                frame_range.for_each(|frame| {
                    let flags = PageTableFlags::PRESENT
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::NO_CACHE;
                    unsafe {
                        match m.identity_map(frame, flags, a) {
                            Ok(r) => {
                                r.flush();
                                MUSE.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(e) => match e {
                                MapToError::PageAlreadyMapped(_) => (),
                                MapToError::FrameAllocationFailed => panic!(
                                    "Cannot map frame at addr {:X} of size {}: no more frames",
                                    frame.clone().start_address(),
                                    frame.size()
                                ),
                                MapToError::ParentEntryHugePage => (),
                            },
                        }
                    }
                });
            }
            _ => panic!("Memory allocator or frame allocator are not set"),
        }
        SMUSE.fetch_add(end - start, Ordering::Relaxed);
        true
    } else {
        false
    }
}

pub fn free_range(start: u64, end: u64) {
    debug!(
        "Freeing memaddr {:X}h-{:X}h ({} bytes)",
        start,
        end,
        end - start
    );
    let mut mapper = MAPPER.lock();
    match mapper.as_mut() {
        Some(m) => {
            let page_range: PageRangeInclusive<Size4KiB> = {
                let start = VirtAddr::new(start);
                let end = VirtAddr::new(end);
                let start_page = Page::containing_address(start);
                let end_page = Page::containing_address(end);
                Page::range_inclusive(start_page, end_page)
            };
            page_range.for_each(|page| match m.unmap(page) {
                Ok((_, r)) => {
                    r.flush();
                    MUSE.fetch_sub(1, Ordering::Relaxed);
                }
                Err(e) => warn!(
                    "Cannot unmap physical memory address range {:X}h-{:X}h: {:#?}",
                    start, end, e
                ),
            });
        }
        _ => panic!("Memory allocator or frame allocator are not set"),
    }
    SMUSE.fetch_sub(end - start, Ordering::Relaxed);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub region_type: MemoryRegionType,
}

pub fn init_memory_map(map: &'static MemoryMap) {
    let mut mmap = MMAP.write();
    map.iter().for_each(|region| {
        mmap.push(MemoryRegion {
            start: region.range.start_addr(),
            end: region.range.end_addr(),
            region_type: region.region_type,
        });
        STOTAL.fetch_add(
            region.range.end_addr() - region.range.start_addr(),
            Ordering::Relaxed,
        );
    });
}

pub fn get_free_addr(size: u64) -> u64 {
    let mut rng = ADDRRNG.lock();
    let region_range: MiniVec<Range<u64>> = MMAP
        .read()
        .iter()
        .filter(|r| r.region_type == MemoryRegionType::Usable)
        .map(|r| r.start..r.end)
        .collect();
    let mut pos = rng.next_u64().wrapping_mul(0x7ABD).wrapping_add(0x1B0F)
        % region_range.iter().map(|r| r.end).max().unwrap()
        - size;
    loop {
        let maxpos = pos + size;
        if region_range.iter().filter(|r| r.contains(&maxpos)).count() > 0 {
            break;
        }
        pos = rng.next_u64().wrapping_mul(0x7ABD).wrapping_add(0x1B0F)
            % region_range.iter().map(|r| r.end).max().unwrap()
            - size;
    }
    let mut addr = pos;
    if addr.get_bits(47..64) != 0 {
        addr.set_bits(47..64, 0);
    }
    addr
}
