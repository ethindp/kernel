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
use core::panic::PanicInfo;
use log::*;
use slab_allocator_rs::*;
use stivale_boot::v2::*;
use x86_64::instructions::random::RdRand;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static LOGGER: Logger = Logger;
const MAX_HEAP_SIZE: u64 = 1024 * 8192;
const MAX_STACK_SIZE: usize = 1024 * 256;
#[repr(C, align(4))]
struct Stack([u8; MAX_STACK_SIZE]);
static STACK: Stack = Stack([0; MAX_STACK_SIZE]);

// Tags
static VIDEO_TAG: StivaleAnyVideoTag = StivaleAnyVideoTag::new()
    .preference(0)
    .next(&FB_TAG as *const StivaleFramebufferHeaderTag as *const ());
static FB_TAG: StivaleFramebufferHeaderTag = StivaleFramebufferHeaderTag::new()
    .framebuffer_bpp(32)
    .next(&TERMINAL_TAG as *const StivaleTerminalHeaderTag as *const ());
static TERMINAL_TAG: StivaleTerminalHeaderTag =
    StivaleTerminalHeaderTag::new().next(&SMP_TAG as *const StivaleSmpHeaderTag as *const ());
static SMP_TAG: StivaleSmpHeaderTag = StivaleSmpHeaderTag::new()
    .flags(StivaleSmpHeaderTagFlags::X2APIC)
    .next(&LVL5_PG_TAG as *const Stivale5LevelPagingHeaderTag as *const ());
static LVL5_PG_TAG: Stivale5LevelPagingHeaderTag = Stivale5LevelPagingHeaderTag::new()
    .next(&UNMAP_NULL_TAG as *const StivaleUnmapNullHeaderTag as *const ());
static UNMAP_NULL_TAG: StivaleUnmapNullHeaderTag = StivaleUnmapNullHeaderTag::new();

#[link_section = ".stivale2hdr"]
#[used]
static BOOT_LOADER_HEADER: StivaleHeader = StivaleHeader::new()
    .stack(STACK.0.as_ptr_range().end)
    .flags((1 << 1) | (1 << 2) | (1 << 3) | (1 << 4))
    .tags(&VIDEO_TAG as *const StivaleAnyVideoTag as *const ());

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
pub extern "C" fn _start(boot_info: &'static StivaleStruct) -> ! {
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
    info!(
        "Booted with {} v. {}",
        boot_info.bootloader_brand(),
        boot_info.bootloader_version()
    );
    info!("Initialization started");
    info!("Initializing interrupt subsystem");
    libk::interrupts::init_idt();
    libk::gdt::init();
    info!("Initializing memory manager");
    let mmap = boot_info
        .memory_map()
        .expect("Bootloader did not provide a memory map!");
    let rsdp = boot_info
        .rsdp()
        .expect("Bootloader did not provide an RSDP address!");
    info!(
        "Stack at addr {:p}, {:p}, {:X}",
        &STACK.0,
        &STACK,
        BOOT_LOADER_HEADER.get_stack() as u64
    );
    libk::memory::init_memory_map(mmap.as_slice(), rsdp.rsdp);
    let vmap = boot_info
        .vmap()
        .expect("Bootloader did not provide a higher-half physical memory offset!");
    let mut idx = usize::MAX;
    let addrs = loop {
        idx = idx.wrapping_add(1);
        let entry = mmap.iter().nth(idx);
        if entry.is_none() {
            break (0, 0);
        }
        let entry = entry.unwrap();
        if entry.entry_type() != StivaleMemoryMapEntryType::Usable {
            continue;
        }
        if entry.length > MAX_HEAP_SIZE {
            let base = entry.base;
            break (base, base + MAX_HEAP_SIZE);
        }
    };
    if addrs == (0, 0) {
        panic!("Can't find a memory region for the heap!");
    }
    libk::memory::init(vmap.address, addrs.0, MAX_HEAP_SIZE);
    unsafe {
        ALLOCATOR.init(addrs.0 as usize, MAX_HEAP_SIZE as usize);
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
