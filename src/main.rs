// SPDX-License-Identifier: MPL-2.0
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(proc_macro_hygiene)]
#![feature(asm)]
#![forbid(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    deprecated_in_future,
    explicit_outlives_requirements,
    indirect_structural_match,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    private_doc_tests,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    semicolon_in_expressions_from_macros,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unaligned_references,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    variant_size_differences,
    warnings,
    box_pointers
)]
#![forbid(clippy::all)]
extern crate alloc;
mod graphics;
use bit_field::BitField;
use bootloader::boot_info::*;
use bootloader::*;
use core::arch::x86_64::{__cpuid, __cpuid_count};
use core::panic::PanicInfo;
use heapless::String;
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
fn kmain(boot_info: &'static mut BootInfo) -> ! {
    x86_64::instructions::interrupts::disable();
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
            let mut buf: String<12> = String::new();
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
                let mut buf: String<128> = String::new();
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
    info!("Initializing memory region list");
    libk::memory::init_memory_map(
        &boot_info.memory_regions,
        boot_info.rsdp_addr.into_option().unwrap(),
    );
    info!("Loading descriptor tables and enabling interrupts");
    libk::gdt::init();
    libk::interrupts::init_idt();
    info!("Initializing virtual memory manager");
    let rdrand = RdRand::new().unwrap();
    let mut start_addr: u64 = 0x0100_0000_0000 + rdrand.get_u64().unwrap();
    start_addr.set_bits(47..64, 0);
    let mut end_addr = start_addr + MAX_HEAP_SIZE;
    end_addr.set_bits(47..64, 0);
    libk::memory::init(
        boot_info.physical_memory_offset.into_option().unwrap(),
        start_addr,
        MAX_HEAP_SIZE,
    );
    info!("Configuring interrupt controller");
    libk::interrupts::init_ic();
    info!("Enabling interrupts");
    x86_64::instructions::interrupts::enable();
    info!("Initializing internal heap allocator");
    unsafe {
        ALLOCATOR
            .lock()
            .init(start_addr as usize, (end_addr - start_addr) as usize);
    }
    info!("firmware-provided memory map:");
    for region in boot_info.memory_regions.iter() {
        info!(
            "[{:X}-{:X}]: {}",
            region.start,
            region.end,
            match region.kind {
                MemoryRegionKind::Usable => "free",
                MemoryRegionKind::UnknownUefi(kind) => match kind {
                    0 => "reserved",
                    1 => "loader code",
                    2 => "loader data",
                    3 => "boot services code",
                    4 => "boot services data",
                    5 => "runtime services code",
                    6 => "runtime services data",
                    8 => "unusable",
                    9 => "acpi reclaimable",
                    10 => "acpi non-volatile",
                    11 => "mmio",
                    12 => "port mmio",
                    13 => "pal code",
                    14 => "free nvm",
                    _ => "unknown uefi",
                },
                MemoryRegionKind::UnknownBios(_) => "unknown bios",
                MemoryRegionKind::Bootloader => "bootloader",
                _ => "Unknown",
            }
        );
    }
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
