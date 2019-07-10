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
use bootloader::bootinfo::*;
use bootloader::*;
use core::panic::PanicInfo;
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
    printkln!("Boot offset is {:X}", boot_info.physical_memory_offset);
    kernel::memory::init(boot_info.physical_memory_offset, &boot_info.memory_map);
    let start_addr: u64 = 0x100000000000;
    let mut end_addr = start_addr + 1 * 1048576;
    while ((end_addr - start_addr) % 32768) != 0 {
        end_addr -= 1;
    }
    unsafe {
        ALLOCATOR.init(start_addr as usize, (end_addr - start_addr) as usize);
    }
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
