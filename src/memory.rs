// SPDX-License-Identifier: MPL-2.0
use crate::printkln;
use alloc::vec::Vec as AllocatedVec;
use bootloader::bootinfo::*;
use lazy_static::lazy_static;
use log::*;
use spin::{Mutex, RwLock};
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
static ref MMAP: RwLock<AllocatedVec<FreeMemoryRegion>> = RwLock::new(AllocatedVec::new());
}

/// Initializes a memory heap for the global memory allocator. Requires a PMO to start with.
unsafe fn init_mapper(physical_memory_offset: u64) -> OffsetPageTable<'static> {
    // Get active L4 table
    trace!(
        "mem: Retrieving active L4 table with memoffset {:X}",
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
) -> Result<(), MapToError<Size4KiB>> {
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
    for page in page_range {
        let frame = match frame_allocator.allocate_frame() {
            Some(f) => f,
            None => panic!("Can't allocate frame!"),
        };
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let frame2 = frame;
        unsafe {
            match mapper.map_to(page, frame, flags, frame_allocator) {
                Ok(f) => f.flush(),
                Err(e) => panic!(
                    "Cannot allocate frame range {:X}h-{:X}h: {:?}",
                    frame2.start_address().as_u64(),
                    frame2.start_address().as_u64() + frame2.size(),
                    e
                ),
            }
        }
    }
    Ok(())
}

/// Allocates a paged heap with the specified permissions.
/// Possible permissions are:
/// * Writable (W): controls whether writes to the mapped frames are allowed. If this bit is unset in a level 1 page table entry, the mapped frame is read-only.
///     If this bit is unset in a higher level page table entry the complete range of mapped pages is read-only.
/// * User accessible (UA): controls whether accesses from userspace (i.e. ring 3) are permitted.
/// * Write-through (WT): if this bit is set, a "write-through" policy is used for the cache, else a "write-back" policy is used.
/// * No cache (NC): Disables caching for this memory page.
/// * Huge page (HP): specifies that the entry maps a huge frame instead of a page table. Only allowed in P2 or P3 tables.
/// * Global (G): indicates that the mapping is present in all address spaces, so it isn't flushed from the TLB on an address space switch.
/// * bits 9, 10, 11, and 52-62: available to the OS, can be used to store additional data, e.g. custom flags.
/// * No execute (NX): forbid code execution from the mapped frames. Can be only used when the no-execute page protection feature is enabled in the EFER register.
fn allocate_paged_heap_with_perms(
    start: u64,
    size: u64,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    permissions: PageTableFlags,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(start);
        let heap_end = heap_start + size - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    for page in page_range {
        let frame = match frame_allocator.allocate_frame() {
            Some(f) => f,
            None => panic!("Can't allocate frame!"),
        };
        let frame2 = frame;
        unsafe {
            match mapper.map_to(page, frame, permissions, frame_allocator) {
                Ok(f) => f.flush(),
                Err(e) => panic!(
                    "Cannot allocate frame range {:X}h-{:X}h: {:?}",
                    frame2.start_address().as_u64(),
                    frame2.start_address().as_u64() + frame2.size(),
                    e
                ),
            }
        }
    }
    Ok(())
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
    pos: usize,
}

impl GlobalFrameAllocator {
    pub fn init(memory_map: &'static MemoryMap) -> Self {
        GlobalFrameAllocator { memory_map, pos: 0 }
    }
}

unsafe impl FrameAllocator<Size4KiB> for GlobalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.pos += 1;
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
            .nth(self.pos)
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
        (Some(m), Some(a)) => match allocate_paged_heap(start_addr, end_addr - start_addr, m, a) {
            Ok(()) => (),
            Err(e) => panic!("Cannot allocate primary heap: {:?}", e),
        },
        _ => panic!("Cannot acquire mapper or frame allocator lock!"),
    }
}

pub fn allocate_heap(start: u64, size: u64) {
    let mut mapper = MAPPER.lock();
    let mut allocator = FRAME_ALLOCATOR.lock();
    match (mapper.as_mut(), allocator.as_mut()) {
        (Some(m), Some(a)) => {
            allocate_paged_heap(start, size, m, a).unwrap();
        }
        _ => panic!("Cannot acquire mapper or frame allocator lock!"),
    }
}

pub fn allocate_heap_with_perms(start: u64, size: u64, perms: PageTableFlags) {
    let mut mapper = MAPPER.lock();
    let mut allocator = FRAME_ALLOCATOR.lock();
    match (mapper.as_mut(), allocator.as_mut()) {
        (Some(m), Some(a)) => {
            allocate_paged_heap_with_perms(start, size, m, a, perms).unwrap();
        }
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
            for page in page_range {
                let frame = match a.allocate_frame() {
                    Some(f) => f,
                    None => panic!("Can't allocate frame!"),
                };
                let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
                unsafe {
                    match m.map_to(page, frame, flags, a) {
                        Ok(r) => r.flush(),
                        Err(_) => {
                            m.unmap(page).unwrap().1.flush();
                            m.map_to(page, frame, flags, a).unwrap().flush();
                        }
                    }
                }
            }
        }
        _ => panic!("Memory allocator or frame allocator are not set"),
    }
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
            for page in page_range {
                let frame = match a.allocate_frame() {
                    Some(f) => f,
                    None => panic!("Can't allocate frame!"),
                };
                unsafe {
                    match m.map_to(page, frame, permissions, a) {
                        Ok(r) => r.flush(),
                        Err(_) => {
                            m.unmap(page).unwrap().1.flush();
                            m.map_to(page, frame, permissions, a).unwrap().flush();
                        }
                    }
                }
            }
        }
        _ => panic!("Memory allocator or frame allocator are not set"),
    }
}

pub fn allocate_phys_range(start: u64, end: u64) {
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
            for frame in frame_range {
                let flags =
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE;
                unsafe {
                    match m.identity_map(frame, flags, a) {
                        Ok(r) => r.flush(),
                        Err(_) => {
                            let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(
                                frame.start_address().as_u64(),
                            ));
                            m.unmap(page).unwrap().1.flush();
                            m.identity_map(frame, flags, a).unwrap().flush();
                        }
                    }
                }
            }
        }
        _ => panic!("Memory allocator or frame allocator are not set"),
    }
}

pub fn free_range(start: u64, end: u64) {
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
            for page in page_range {
                match m.unmap(page) {
                        Ok((_, r)) => r.flush(),
                        Err(e) => printkln!(
                            "Kernel: warning: Cannot unmap physical memory address range {:X}h-{:X}h: {:#?}",
                            start, end, e
                        ),
                    }
            }
        }
        _ => panic!("Memory allocator or frame allocator are not set"),
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FreeMemoryRegion {
    pub start: usize,
    pub end: usize,
}

pub fn init_free_memory_map(map: &'static MemoryMap) {
    let mut mmap = MMAP.write();
    for region in map
        .iter()
        .filter(|r| r.region_type == MemoryRegionType::Usable)
    {
        let mut mr = FreeMemoryRegion::default();
        mr.start = region.range.start_addr() as usize;
        mr.end = region.range.end_addr() as usize;
        mmap.push(mr);
    }
}
