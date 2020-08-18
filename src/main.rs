// SPDX-License-Identifier: MPL-2.0
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
use core::arch::x86_64::{__cpuid, __cpuid_count};
use core::panic::PanicInfo;
use heapless::{consts::*, String};
use linked_list_allocator::*;
use x86_64::instructions::random::RdRand;
use log::*;

entry_point!(kmain);
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static LOGGER: Logger = Logger;

// Panic handler
#[panic_handler]
fn panic(panic_information: &PanicInfo) -> ! {
    printkln!("Fatal error: {}", panic_information);
    kernel::idle_forever();
}

// Kernel entry point
fn kmain(boot_info: &'static BootInfo) -> ! {
set_logger(&LOGGER).unwrap();
set_max_level(LevelFilter::Debug);
    if RdRand::new().is_none() {
        printkln!("Error: rdrand is not supported on this system, but rdrand is required");
        kernel::idle_forever();
    }
    printkln!("Init: kernel initialization started");
    printkln!("init: CPU identification and configuration initiated");
    unsafe {
        let mut res = __cpuid_count(0, 0);
        let vs = {
            let mut buf: String<U12> = String::new();
            let ebx = u32::to_le_bytes(res.ebx);
            let edx = u32::to_le_bytes(res.edx);
            let ecx = u32::to_le_bytes(res.ecx);
            // Reassemble vendor string
            for i in ebx.iter() {
                buf.push(*i as char).unwrap();
            }
            for i in edx.iter() {
                buf.push(*i as char).unwrap();
            }
            for i in ecx.iter() {
                buf.push(*i as char).unwrap();
            }
            buf
        };
        res = __cpuid(0x80000000);
        if res.eax > 0x80000000 {
            let bs = {
                let mut buf: String<U128> = String::new();
                for i in 0x80000002 as u32..0x80000005 as u32 {
                    let res = __cpuid(i as u32);
                    for i in u32::to_le_bytes(res.eax).iter() {
                        buf.push(*i as char).unwrap();
                    }
                    for i in u32::to_le_bytes(res.ebx).iter() {
                        buf.push(*i as char).unwrap();
                    }
                    for i in u32::to_le_bytes(res.ecx).iter() {
                        buf.push(*i as char).unwrap();
                    }
                    for i in u32::to_le_bytes(res.edx).iter() {
                        buf.push(*i as char).unwrap();
                    }
                }
                buf
            };
            printkln!("init: detected {} {} processor", vs, bs);
        } else {
            printkln!("init: detected {} processor", vs);
        }
    }
    printkln!("init: Configuring processor");
    printkln!("Init: Locating kernel heap area");
    let rdrand = RdRand::new().unwrap();
    let mut start_addr: u64 = 0x0100_0000_0000 + rdrand.get_u64().unwrap();
    if start_addr.get_bits(48..64) > 0 {
        start_addr.set_bits(48..64, 0);
    }
    let end_addr = start_addr + 1_048_576;
    printkln!("init: initializing memory manager");
    kernel::memory::init(
        boot_info.physical_memory_offset,
        &boot_info.memory_map,
        start_addr,
        1048576,
    );
    printkln!("Init: enabling interrupts, first stage");
    kernel::interrupts::init_stage1();
    printkln!("init: Initializing global heap allocator");
    unsafe {
        ALLOCATOR
            .lock()
            .init(start_addr as usize, (end_addr - start_addr) as usize);
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
                MemoryRegionType::AcpiReclaimable => "ACPI, reclaimable",
                MemoryRegionType::AcpiNvs => "ACPI, NVS",
                MemoryRegionType::BadMemory => "bad",
                MemoryRegionType::Kernel => "Kernel area",
                MemoryRegionType::KernelStack => "Kernel stack",
                MemoryRegionType::PageTable => "Page table",
                MemoryRegionType::Bootloader => "Boot loader",
                MemoryRegionType::FrameZero => "NULL",
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

struct Logger;

impl Log for Logger {
fn enabled(&self, metadata: &Metadata) -> bool {
true
}

fn log(&self, record: &Record) {
if self.enabled(record.metadata()) {
use kernel::printkln;
printkln!("[{}] [{}] {}", record.level(), record.target(), record.args());
} else {
return;
}
}

fn flush(&self) { }
}

