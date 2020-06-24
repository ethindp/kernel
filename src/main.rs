#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(proc_macro_hygiene)]
#![feature(asm)]
#![feature(const_in_array_repeat_expressions)]
#![allow(dead_code)]
extern crate alloc;
extern crate uart_16550;
extern crate x86_64;
mod memory;
mod vga;
use bit_field::BitField;
use bootloader::bootinfo::*;
use bootloader::*;
use core::panic::PanicInfo;
use buddy_system_allocator::LockedHeap;
use x86_64::instructions::random::RdRand;

entry_point!(kmain);
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::new();

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
    printkln!("Init: kernel initialization started");
    printkln!("Init: Locating kernel heap area, size 8 MB");
    let rdrand = RdRand::new().unwrap();
    let mut start_addr: u64 = 0x0100_0000_0000 + rdrand.get_u64().unwrap();
    if start_addr.get_bits(48..64) > 0 {
        start_addr.set_bits(48..64, 0);
    }
    let mut end_addr = start_addr + 8 * 1_048_576;
    while ((end_addr - start_addr) % 32768) != 0 {
        end_addr -= 1;
    }
    printkln!("init: initializing memory manager");
    kernel::memory::init(
        boot_info.physical_memory_offset,
        &boot_info.memory_map,
        start_addr,
    );
    printkln!("Init: enabling interrupts, first stage");
    kernel::interrupts::init_stage1();
    printkln!("init: Initializing global heap allocator");
    unsafe {
        ALLOCATOR.lock().init(start_addr as usize, (end_addr - start_addr) as usize);
    }
    printkln!("init: firmware-provided memory map:");
    for region in boot_info.memory_map.iter() {
        printkln!(
            "[{:X}-{:X}] [size {}]: {}",
            region.range.start_addr(),
            region.range.end_addr(),
            region.range.end_addr() - region.range.start_addr(),
            match region.region_type {
                MemoryRegionType::Usable => "free",
                MemoryRegionType::InUse => "sw-reserved",
                MemoryRegionType::Reserved => "hw-reserved",
                MemoryRegionType::AcpiReclaimable => "ACPI reclaimable",
                MemoryRegionType::AcpiNvs => "ACPI NVS",
                MemoryRegionType::BadMemory => "bad",
                MemoryRegionType::Kernel => "reserved",
                MemoryRegionType::KernelStack => "reserved",
                MemoryRegionType::PageTable => "reserved",
                MemoryRegionType::Bootloader => "reserved",
                MemoryRegionType::FrameZero => "null",
                MemoryRegionType::Empty => "empty",
                MemoryRegionType::BootInfo => "Boot information",
                MemoryRegionType::Package => "pkg",
                _ => "unknown",
            }
        );
    }
    kernel::memory::init_free_memory_map(&boot_info.memory_map);
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
