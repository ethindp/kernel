// SPDX-License-Identifier: MPL-2.0
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(proc_macro_hygiene)]
#![feature(asm)]
#![allow(dead_code)]
#![forbid(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    deprecated_in_future,
    explicit_outlives_requirements,
    indirect_structural_match,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    non_ascii_idents,
    private_doc_tests,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unaligned_references,
    unreachable_pub,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    variant_size_differences
)]
#![deny(
    warnings,
    missing_copy_implementations,
    missing_debug_implementations,
    box_pointers
)]
#![forbid(clippy::all)]
extern crate alloc;
mod vga;
use bit_field::BitField;
use bootloader::bootinfo::*;
use bootloader::*;
use core::arch::x86_64::{__cpuid, __cpuid_count};
use core::panic::PanicInfo;
use heapless::{consts::*, String};
use linked_list_allocator::*;
use log::*;
use x86_64::instructions::random::RdRand;

entry_point!(kmain);
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static LOGGER: Logger = Logger;
const MAX_HEAP_SIZE: u64 = 4096 * 2048;

include!(concat!(env!("OUT_DIR"), "/verinfo.rs"));
include!(concat!(env!("OUT_DIR"), "/build_details.rs"));

// Panic handler
#[panic_handler]
fn panic(panic_information: &PanicInfo) -> ! {
    error!("{}", panic_information);
    libk::idle_forever();
}

// Kernel entry point
fn kmain(boot_info: &'static BootInfo) -> ! {
    set_logger(&LOGGER).unwrap();
    if cfg!(debug_assertions) {
        set_max_level(LevelFilter::Trace);
    } else {
        set_max_level(LevelFilter::Info);
    }
    if RdRand::new().is_none() {
        error!("rdrand is not supported on this system, but rdrand is required");
        libk::idle_forever();
    }

    info!(
        "Kernel, v. {}",
        if let Some(version) = VERSION {
            version
        } else {
            "Unknown"
        }
    );
    info!("Compiled with {}, {} build", RUSTC_VER, PROFILE.unwrap());
    info!("Initialization started");
    info!("CPU identification and configuration initiated");
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
                for i in 0x80000002u32..0x80000005u32 {
                    let res = __cpuid(i);
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
            info!("Detected processor: {} {}", vs, bs);
        } else {
            info!("Detected processor: {}", vs);
        }
    }
    info!("Configuring processor");
    info!("Locating kernel heap area");
    let rdrand = RdRand::new().unwrap();
    let mut start_addr: u64 = 0x0100_0000_0000 + rdrand.get_u64().unwrap();
    start_addr.set_bits(47..64, 0);
    let mut end_addr = start_addr + MAX_HEAP_SIZE;
    end_addr.set_bits(47..64, 0);
    info!("Initializing memory manager");
    libk::memory::init(
        boot_info.physical_memory_offset,
        &boot_info.memory_map,
        start_addr,
        MAX_HEAP_SIZE,
    );
    info!("Enabling interrupts, first stage");
    libk::interrupts::init_stage1();
    info!("Initializing global heap allocator");
    unsafe {
        ALLOCATOR
            .lock()
            .init(start_addr as usize, (end_addr - start_addr) as usize);
    }
    info!("init: firmware-provided memory map:");
    for region in boot_info.memory_map.iter() {
        info!(
            "[{:X}-{:X}] [size {:X}]: {}",
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
                MemoryRegionType::Kernel => "reserved by kernel",
                MemoryRegionType::KernelStack => "reserved by kernel",
                MemoryRegionType::PageTable => "reserved by kernel",
                MemoryRegionType::Bootloader => "reserved by boot loader",
                MemoryRegionType::FrameZero => "NULL",
                MemoryRegionType::Empty => "empty",
                MemoryRegionType::BootInfo => "reserved by boot information",
                MemoryRegionType::Package => "pkg",
                _ => "unknown",
            }
        );
    }
    libk::memory::init_memory_map(&boot_info.memory_map);
    libk::init();
    libk::idle_forever();
}

// Memory allocation error handler
// For now, we just print how much was needed and its alignment.
#[alloc_error_handler]
fn handle_alloc_failure(layout: core::alloc::Layout) -> ! {
    panic!(
        "Cannot allocate memory of min. size {} and min. alignment of {}, layout: {:?}",
        layout.size(),
        layout.align(),
        layout
    )
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            printkln!(
                "[{}] [{}] {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
