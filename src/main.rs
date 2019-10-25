#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(proc_macro_hygiene)]
#![feature(asm)]
extern crate alloc;
extern crate uart_16550;
extern crate x86_64;
mod memory;
mod ui;
mod vga;
use bit_field::BitField;
use bootloader::bootinfo::*;
use bootloader::*;
use core::panic::PanicInfo;
use slab_allocator::LockedHeap;
use x86_64::registers::control::Cr0;

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
    kernel::memory::init(boot_info.physical_memory_offset, &boot_info.memory_map);
    let start_addr: u64 = 0x100000000000;
    let mut end_addr = start_addr + 1 * 1048576;
    while ((end_addr - start_addr) % 32768) != 0 {
        end_addr -= 1;
    }
    unsafe {
        ALLOCATOR.init(start_addr as usize, (end_addr - start_addr) as usize);
    }
    printkln!("Enabling SSE");
    let mut flags = Cr0::read_raw();
    flags.set_bit(2, false);
    flags.set_bit(1, true);
    flags.set_bit(9, true);
    flags.set_bit(10, true);
    unsafe {
        Cr0::write_raw(flags);
    }
    // For now, we must use inline ASM here
    let mut cr4: u64;
    unsafe {
        asm!("mov %cr4, $0" : "=r" (cr4));
    }
    cr4.set_bit(9, true);
    cr4.set_bit(10, true);
    unsafe {
        asm!("mov $0, %cr4" :: "r" (cr4) : "memory");
    }
    printkln!("SSE enabled");
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
