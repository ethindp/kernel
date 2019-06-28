#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
extern crate alloc;
extern crate raw_cpuid;
extern crate uart_16550;
extern crate x86_64;
mod memory;
mod ui;
mod vga;
use bit_field::BitField;
use bootloader::bootinfo::*;
use bootloader::*;
use core::panic::PanicInfo;
use cpuio::*;
use slab_allocator::LockedHeap;
use x86_64::{structures::paging::{FrameAllocator, MappedPageTable, MapperAllSizes, PageTable, PageTableFlags, Mapper, PhysFrame, Size4KiB, Page}, PhysAddr, VirtAddr};

entry_point!(kmain);
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
// Panic handler
#[panic_handler]
fn panic(panic_information: &PanicInfo) -> ! {
    printkln!("Fatal error: {}", panic_information);
    kernel::idle_forever();
}

// Kernel entry point
fn kmain(boot_info: &'static BootInfo) -> ! {
    printkln!("Loading kernel");
    // Set up the page mapper and global frame allocator (GFA).
    let mut mapper = unsafe { kernel::memory::init(boot_info.physical_memory_offset) };
    let mut frame_allocator =
        unsafe { kernel::memory::GlobalFrameAllocator::init(&boot_info.memory_map) };
    // Determine the size of our tiny kernel heap
    let mut start_addr: u64 = 0u64;
let mut end_addr = 0u64;
let mut actual_end_addr = 0u64;
    for region in boot_info.memory_map.iter() {
        if region.region_type == MemoryRegionType::Usable {
            start_addr = boot_info.physical_memory_offset + region.range.start_addr();
end_addr = boot_info.physical_memory_offset + region.range.end_addr();
actual_end_addr = boot_info.physical_memory_offset + region.range.end_addr();
while ((end_addr - start_addr) % 32768) != 0 {
end_addr = end_addr - 1;
}
            break;
        }
    }
printkln!("Mapping a memory area of size {} bytes into page table with {} bytes unused", end_addr - start_addr, actual_end_addr - end_addr);
kernel::memory::init_heap(start_addr, end_addr-start_addr, &mut mapper, &mut frame_allocator);
unsafe { ALLOCATOR.init(start_addr as usize, (end_addr - start_addr) as usize); }
    // Load the remaining subsystems
    kernel::init();
    printkln!("Kernel init done!");
    // Initialize the TUI and transfer control to it
    ui::init();
    // We should *never* reach this point.
    kernel::idle_forever();
}

// Memory allocation error handler
// For now, we just print how much was needed and its alignment.
#[alloc_error_handler]
fn handle_alloc_failure(layout: core::alloc::Layout) -> ! {
    panic!(
        "Cannot allocate memory of min. size {} and min. alignment of {}",
        layout.size(),
        layout.align()
    )
}
