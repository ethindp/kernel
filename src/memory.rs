// This code was almost directly written from Writing an OS in Rust by Phil-op on github. We need to improve it though and get the kernel to fully use paging. (It wasn't written by phil-op, but by me, with a few modifications to fit the kernel.)
use bootloader::bootinfo::*;
use x86_64::registers::control::*;
use x86_64::{
    structures::paging::{
        FrameAllocator, MappedPageTable, MapperAllSizes, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};
pub unsafe fn init(physical_memory_offset: u64) -> impl MapperAllSizes {
    let (level_4_table, _) = get_active_l4_table(physical_memory_offset);
    let phys_to_virt = move |frame: PhysFrame| -> *mut PageTable {
        let phys = frame.start_address().as_u64();
        let virt = VirtAddr::new(phys + physical_memory_offset);
        virt.as_mut_ptr()
    };
    MappedPageTable::new(level_4_table, phys_to_virt)
}

unsafe fn get_active_l4_table(physical_memory_offset: u64) -> (&'static mut PageTable, Cr3Flags) {
    let (table_frame, flags) = Cr3::read();
    let phys = table_frame.start_address();
    let virt = VirtAddr::new(phys.as_u64() + physical_memory_offset);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    (&mut *page_table_ptr, flags)
}

pub struct GlobalFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl GlobalFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        GlobalFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn iter_usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions =
            regions.filter(|region| region.region_type == MemoryRegionType::Usable);
        let address_ranges =
            usable_regions.map(|region| region.range.start_addr()..region.range.end_addr());
        let frame_addresses = address_ranges.flat_map(|region| region.step_by(4096));
        frame_addresses.map(|address| PhysFrame::containing_address(PhysAddr::new(address)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for GlobalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.iter_usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
