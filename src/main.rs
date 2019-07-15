#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(proc_macro_hygiene)] 
#![feature(asm)] 
extern crate alloc;
extern crate raw_cpuid;
extern crate uart_16550;
extern crate x86_64;
extern crate rusty_asm;
mod memory;
mod ui;
mod vga;
use bootloader::bootinfo::*;
use bootloader::*;
use core::panic::PanicInfo;
use slab_allocator::LockedHeap;
use raw_cpuid::*;
use x86_64::registers::control::*;
use bit_field::BitField;
use rusty_asm::rusty_asm;

unsafe fn configure_sse_cr4() {
rusty_asm! {
asm("volatile") {
r#"mov %cr4, %rax
or 3 << 9, %ax
mov %rax, %cr4"#
}
}
}

unsafe fn configure_fpu_cr4() {
rusty_asm! {
asm("volatile") {
r#"mov %cr4, %rax
or 9, %ax
or 10, %ax
or 18, %ax
mov %rax, %cr4
fninit"#
}
}
}

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
    kernel::init();
let cpu = CpuId::new();
match cpu.get_feature_info() {
Some(features) => {
if features.has_sse() {
printkln!("Detected SSE support; enabling");
let mut cr0 = Cr0::read_raw();
cr0.set_bit(2, false);
cr0.set_bit(1, true);
unsafe {
Cr0::write_raw(cr0);
configure_sse_cr4();
}
printkln!("SSE enabled!");
}
if features.has_fpu() {
printkln!("Detected FPU; enabling");
let mut cr0 = Cr0::read_raw();
cr0.set_bit(2, false);
cr0.set_bit(5, true);
cr0.set_bit(1, true);
unsafe {
Cr0::write_raw(cr0);
configure_fpu_cr4();
}
}
},
None => (),
}
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
