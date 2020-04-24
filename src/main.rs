#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(proc_macro_hygiene)]
#![feature(asm)]
#![feature(const_in_array_repeat_expressions)]
#![allow(dead_code)]
#![deny(clippy::all)]
extern crate alloc;
extern crate uart_16550;
extern crate x86_64;
mod memory;
mod vga;
use bootloader::bootinfo::*;
use bootloader::*;
use core::panic::PanicInfo;
use linked_list_allocator::LockedHeap;
use x86_64::instructions::random::RdRand;

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
    if RdRand::new().is_none() {
        printkln!("Error: rdrand is not supported on this system, but rdrand is required");
        kernel::idle_forever();
    }
    printkln!("Loading kernel");
    let start_addr: u64 = 0x1000_0000_0000;
    let mut end_addr = start_addr + 8 * 1_048_576;
    while ((end_addr - start_addr) % 32768) != 0 {
        end_addr -= 1;
    }
    kernel::memory::init(
        boot_info.physical_memory_offset,
        &boot_info.memory_map,
        start_addr,
    );
    printkln!("Enabling interrupts");
    kernel::gdt::init();
    kernel::interrupts::init();
    unsafe {
        ALLOCATOR
            .lock()
            .init(start_addr as usize, (end_addr - start_addr) as usize);
    }
    kernel::init();
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
