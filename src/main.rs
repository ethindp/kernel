// SPDX-License-Identifier: MPL-2.0
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(proc_macro_hygiene)]
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
use core::panic::PanicInfo;
use log::*;
use slab_allocator_rs::*;
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
#[no_mangle]
fn kmain(boot_info: &'static mut BootInfo) -> ! {
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
    info!("Initializing interrupt subsystem");
    libk::gdt::init();
    libk::interrupts::init_idt();
    libk::interrupts::init_ic();
    info!("Initializing memory management subsystem");
    libk::memory::init_memory_map(
        &boot_info.memory_regions,
        boot_info.rsdp_addr.into_option().unwrap(),
    );
    let rdrand = RdRand::new().unwrap();
    let mut start_addr: u64 = 0x0100_0000_0000_0000 + rdrand.get_u64().unwrap();
    start_addr.set_bits(47..64, 0);
    start_addr.set_bits(0..12, 0);
    let mut end_addr = start_addr + MAX_HEAP_SIZE;
    end_addr.set_bits(47..64, 0);
    end_addr.set_bits(0..12, 0);
    libk::memory::init(
        boot_info.physical_memory_offset.into_option().unwrap(),
        start_addr,
        MAX_HEAP_SIZE,
    );
    unsafe {
        ALLOCATOR.init(start_addr as usize, (end_addr - start_addr) as usize);
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
