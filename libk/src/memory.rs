// SPDX-License-Identifier: MPL-2.0
use bit_field::BitField;
use core::ops::Range;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use heapless::Vec;
use log::*;
use minivec::MiniVec;
use rand_core::{RngCore, SeedableRng};
use rand_hc::Hc128Rng;
use spin::{mutex::ticket::TicketMutex, Lazy, Once};
use stivale_boot::v2::*;
use x86_64::{
    addr::align_up,
    instructions::random::RdRand,
    registers::control::*,
    structures::paging::{
        mapper::MapToError, page::PageRangeInclusive, FrameAllocator, FrameDeallocator, Mapper,
        OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// The page table mapper (PTM) used by the kernel global memory allocator.
static MAPPER: Lazy<TicketMutex<Option<OffsetPageTable<'static>>>> =
    Lazy::new(|| TicketMutex::new(None));
/// The global frame allocator (GFA); works in conjunction with the PTM.
static FRAME_ALLOCATOR: Lazy<TicketMutex<Option<GlobalFrameAllocator>>> =
    Lazy::new(|| TicketMutex::new(None));
static MMAP: Once<Vec<MemoryRegion, 1024>> = Once::new();
static ADDRRNG: Lazy<TicketMutex<Hc128Rng>> = Lazy::new(|| {
    TicketMutex::new({
        let rand = RdRand::new().unwrap();
        let mut seed = [0u8; 32768];
        let seeds = (0..1024)
            .map(|_| rand.get_u64().unwrap().to_ne_bytes())
            .collect::<Vec<[u8; 8], 1024>>();
        seed.iter_mut()
            .zip(seeds.iter().flatten())
            .for_each(|(i, j)| *i = *j);
        Hc128Rng::from_seed(*blake3::hash(&seed).as_bytes())
    })
});
static MUSE: AtomicU64 = AtomicU64::new(0);
static SMUSE: AtomicU64 = AtomicU64::new(0);
static STOTAL: AtomicU64 = AtomicU64::new(0);
static FPOS: AtomicUsize = AtomicUsize::new(0);
static RSDP: AtomicU64 = AtomicU64::new(0);

/// Initializes a memory heap for the global memory allocator. Requires a PMO to start with.
#[cold]
unsafe fn init_mapper(physical_memory_offset: u64) -> OffsetPageTable<'static> {
    // Get active L4 table
    trace!(
        "Retrieving active L4 table with memoffset {:X}",
        physical_memory_offset
    );
    unsafe {
        let (level_4_table, _) = get_active_l4_table(physical_memory_offset);
        // initialize the mapper
        OffsetPageTable::new(
            level_4_table,
            VirtAddr::new_truncate(physical_memory_offset),
        )
    }
}

/// Allocates a paged heap.
#[cold]
pub fn allocate_paged_heap(
    start: u64,
    size: u64,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    perms: Option<PageTableFlags>,
) {
    debug!(
        "Allocating heap in paged memory with start of {:X}, size {:X}",
        start, size
    );
    // Construct a page range
    let page_range = {
        // Calculate start and end
        let heap_start = VirtAddr::new_truncate(start);
        let heap_end = heap_start + size - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    debug!("Page range constructed: {:?}", page_range);
    // Allocate appropriate page frames
    page_range.for_each(|page| {
        debug!(
            "Requesting new page frame for page at addr {:X} with size {:X}",
            page.start_address().as_u64(),
            page.size()
        );
        debug!(
            "Page table indexes: {:?}, {:?}, {:?}, {:?}",
            page.p4_index(),
            page.p3_index(),
            page.p2_index(),
            page.p1_index()
        );
        let frame = match frame_allocator.allocate_frame() {
            Some(f) => f,
            None => panic!("Can't allocate frame!"),
        };
        let flags = if let Some(flags) = perms {
            PageTableFlags::PRESENT | flags
        } else {
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE
        };
        debug!("Requesting mapping of page with flags {:X}", flags);
        unsafe {
            match mapper.map_to(page, frame, flags, frame_allocator) {
                Ok(f) => {
                    debug!("Map complete, flushing TLB");
                    f.flush();
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
    SMUSE.fetch_add(size, Ordering::Relaxed);
}

#[cold]
unsafe fn get_active_l4_table(physical_memory_offset: u64) -> (&'static mut PageTable, Cr3Flags) {
    let (table_frame, flags) = Cr3::read();
    let phys = table_frame.start_address();
    let virt = VirtAddr::new_truncate(phys.as_u64() + physical_memory_offset);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { (&mut *page_table_ptr, flags) }
}

#[derive(Debug, Copy, Clone)]
struct GlobalFrameAllocator;

impl GlobalFrameAllocator {
    /// Initializes the global frame allocator
    #[cold]
    pub(crate) fn init() -> Self {
        FPOS.store(0, Ordering::Relaxed);
        GlobalFrameAllocator {}
    }
}

unsafe impl FrameAllocator<Size4KiB> for GlobalFrameAllocator {
    #[must_use]
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        FPOS.fetch_add(1, Ordering::SeqCst);
        let pos = FPOS.load(Ordering::Relaxed);
        MMAP.wait()
            .iter()
            .filter(|r| r.kind == StivaleMemoryMapEntryType::Usable)
            .map(|r| r.start..r.end)
            .flat_map(|r| r.step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new_truncate(addr)))
            .nth(pos)
    }
}

impl FrameDeallocator<Size4KiB> for GlobalFrameAllocator {
    unsafe fn deallocate_frame(&mut self, _: PhysFrame) {
        FPOS.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Initializes the memory subsystem.
#[cold]
pub fn init(physical_memory_offset: u64, start_addr: u64, size: u64) {
    let mut mapper = MAPPER.lock();
    *mapper = Some(unsafe { init_mapper(physical_memory_offset) });
    let mut allocator = FRAME_ALLOCATOR.lock();
    *allocator = Some(GlobalFrameAllocator::init());
    let end_addr = start_addr + size;
    match (mapper.as_mut(), allocator.as_mut()) {
        (Some(m), Some(a)) => allocate_paged_heap(start_addr, end_addr - start_addr, m, a, None),
        _ => panic!("Memory allocator or page frame allocator failed creation!"),
    }
}

/// Allocates a paged (virtual) contiguous address range within [start, end]. `end` must be >= `start` and vice-versa.
/// If `perms` is not `None`, allows specification of custom privileges for the range. The `P` (present) bit is always set.
pub fn allocate_page_range(start: u64, end: u64, perms: Option<PageTableFlags>) {
    if end < start {
        warn!(
            "attempt to allocate {} with start of {:X} and end of {:X}",
            end - start,
            start,
            end
        );
        return;
    }
    match (MAPPER.lock().as_mut(), FRAME_ALLOCATOR.lock().as_mut()) {
        (Some(m), Some(a)) => {
            let page_range = {
                let start = VirtAddr::new_truncate(start);
                let end = VirtAddr::new_truncate(end);
                let start_page = Page::containing_address(start);
                let end_page = Page::containing_address(end);
                Page::range_inclusive(start_page, end_page)
            };
            page_range.for_each(|page| {
                let frame = match a.allocate_frame() {
                    Some(f) => f,
                    None => panic!("Can't allocate frame!"),
                };
                let flags = if let Some(flags) = perms {
                    PageTableFlags::PRESENT | flags
                } else {
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE
                };
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

/// Allocates a physical memory address range within [start, end]. `end` must be > `start`.
/// If `force` is specified, the allocation will occur even if the range is not marked as usable (free).
/// If `perms` is not `None`, custom permissions can be specified for this memory range. The `P` (present) bit is always set.
pub fn allocate_phys_range(
    start: u64,
    end: u64,
    force: bool,
    perms: Option<PageTableFlags>,
) -> bool {
    if end < start {
        warn!(
            "attempt to allocate {} with start of {:X} and end of {:X}",
            end - start,
            start,
            end
        );
        return false;
    }
    let m = MMAP.get().unwrap();
    let mut ret = true;
    let cnt = m
        .iter()
        .filter(|r| {
            r.kind == StivaleMemoryMapEntryType::Usable
                && (r.start..r.end).contains(&start)
                && (r.start..r.end).contains(&end)
        })
        .count();
    if cnt > 0 || force {
        match (MAPPER.lock().as_mut(), FRAME_ALLOCATOR.lock().as_mut()) {
            (Some(m), Some(a)) => {
                let frame_range = {
                    let start = PhysAddr::new_truncate(start);
                    let end = PhysAddr::new_truncate(end);
                    let start_frame = PhysFrame::<Size4KiB>::containing_address(start);
                    let end_frame = PhysFrame::<Size4KiB>::containing_address(end);
                    PhysFrame::range_inclusive(start_frame, end_frame)
                };
                frame_range.for_each(|frame| {
                    let flags = if let Some(flags) = perms {
                        PageTableFlags::PRESENT | flags
                    } else {
                        PageTableFlags::PRESENT
                            | PageTableFlags::WRITABLE
                            | PageTableFlags::NO_CACHE
                    };
                    unsafe {
                        match m.identity_map(frame, flags, a) {
                            Ok(r) => {
                                r.flush();
                                MUSE.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(e) => match e {
                                MapToError::PageAlreadyMapped(_) => ret = false,
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
        ret
    } else {
        false
    }
}

/// Frees a contiguous range of memory (either virtual or physical). Is a no-op if this range is not allocated. `end` must be > `start`.
pub fn free_range(start: u64, end: u64) -> bool {
    if end < start {
        warn!(
            "attempt to allocate {} with start of {:X} and end of {:X}",
            end - start,
            start,
            end
        );
        return false;
    }
    let mut ret = false;
    let page_range: PageRangeInclusive<Size4KiB> = {
        let start = VirtAddr::new_truncate(start);
        let end = VirtAddr::new_truncate(end);
        let start_page = Page::containing_address(start);
        let end_page = Page::containing_address(end);
        Page::range_inclusive(start_page, end_page)
    };
    match MAPPER.lock().as_mut() {
        Some(m) => {
            page_range.for_each(|page| match m.unmap(page) {
                Ok((_, r)) => {
                    r.flush();
                    MUSE.fetch_sub(1, Ordering::Relaxed);
                    ret = true;
                }
                Err(e) => {
                    warn!(
                        "Cannot unmap physical memory address range {:X}h-{:X}h: {:#?}",
                        start, end, e
                    );
                    ret = false;
                }
            });
        }
        _ => panic!("Page mapper not initialized!"),
    }
    SMUSE.fetch_sub(end - start, Ordering::Relaxed);
    ret
}

#[derive(Clone, Copy, Debug)]
struct MemoryRegion {
    pub(crate) start: u64,
    pub(crate) end: u64,
    pub(crate) kind: StivaleMemoryMapEntryType,
}

/// Initializes the internal memory map.
#[cold]
pub fn init_memory_map(map: &[StivaleMemoryMapEntry], rsdpaddr: u64) {
    info!(
        "Loading free memory region list from memory map at {:p}",
        &map
    );
    MMAP.call_once(|| {
        let mut mmap: Vec<MemoryRegion, 1024> = Vec::new();
        map.iter().for_each(|region| {
            mmap.push(MemoryRegion {
                start: region.base,
                end: region.end_address(),
                kind: region.entry_type(),
            })
            .unwrap();
            STOTAL.fetch_add(region.end_address() - region.base, Ordering::Relaxed);
        });
        mmap
    });
    info!("Discovered {} bytes of RAM", STOTAL.load(Ordering::Relaxed));
    info!("RSDP at {:X}", rsdpaddr);
    RSDP.swap(rsdpaddr, Ordering::Relaxed);
}

/// Attempts to find a random memory address that is free that allows allocations of the given size.
#[no_mangle]
pub extern "C" fn get_free_addr(size: u64) -> u64 {
    let region_range: MiniVec<Range<u64>> = MMAP
        .get()
        .unwrap()
        .iter()
        .filter(|r| r.kind == StivaleMemoryMapEntryType::Usable)
        .map(|r| r.start..r.end)
        .collect();
    let mut addrrng = ADDRRNG.lock();
    let mut pos = addrrng.next_u64().wrapping_mul(0x7ABD).wrapping_add(0x1B0F)
        % region_range.iter().map(|r| r.end).max().unwrap()
        - size;
    loop {
        let maxpos = pos + size;
        if region_range.iter().filter(|r| r.contains(&maxpos)).count() > 0 {
            break;
        }
        pos = addrrng.next_u64().wrapping_mul(0x7ABD).wrapping_add(0x1B0F)
            % region_range.iter().map(|r| r.end).max().unwrap()
            - size;
    }
    let mut addr = pos;
    if addr.get_bits(47..64) != 0 {
        addr.set_bits(47..64, 0);
    }
    addr
}

/// Attempts to find a memory address that allows allocations of the given size. Will automatically align the address to the given alignment before returning.
#[no_mangle]
pub extern "C" fn get_aligned_free_addr(size: u64, alignment: u64) -> u64 {
    let region_range: MiniVec<Range<u64>> = MMAP
        .get()
        .unwrap()
        .iter()
        .filter(|r| r.kind == StivaleMemoryMapEntryType::Usable)
        .map(|r| r.start..r.end)
        .collect();
    let mut addrrng = ADDRRNG.lock();
    let mut pos = align_up(
        addrrng.next_u64().wrapping_mul(0x7ABD).wrapping_add(0x1B0F)
            % region_range.iter().map(|r| r.end).max().unwrap()
            - size,
        alignment,
    );
    loop {
        let maxpos = align_up(pos + size, alignment);
        if region_range.iter().filter(|r| r.contains(&maxpos)).count() > 0 {
            break;
        }
        pos = align_up(
            addrrng.next_u64().wrapping_mul(0x7ABD).wrapping_add(0x1B0F)
                % region_range.iter().map(|r| r.end).max().unwrap()
                - size,
            alignment,
        );
    }
    let mut addr = pos;
    if addr.get_bits(47..64) != 0 {
        addr.set_bits(47..64, 0);
    }
    addr
}

/// Gets the address for the RSDP
#[inline]
pub fn get_rsdp() -> u64 {
    RSDP.load(Ordering::Relaxed)
}
