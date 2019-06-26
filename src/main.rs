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
    let _mapper = unsafe { kernel::memory::init(boot_info.physical_memory_offset) };
    let _frame_allocator =
        unsafe { kernel::memory::GlobalFrameAllocator::init(&boot_info.memory_map) };
    // Determine the size of our tiny kernel heap
    let mut start_addr: u64 = 0;
    let mut end_addr: u64 = 0;
    for region in boot_info.memory_map.iter() {
        if region.region_type == MemoryRegionType::Usable {
            start_addr = boot_info.physical_memory_offset + region.range.start_addr();
            end_addr = boot_info.physical_memory_offset + region.range.end_addr();
            break;
        }
    }
    // Set it as the heap for the allocator to use.
    unsafe {
        ALLOCATOR.init(start_addr as usize, end_addr as usize - start_addr as usize);
    }
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
