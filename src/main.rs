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
use limine::*;
use x86_64::instructions::random::RdRand;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
static LOGGER: Logger = Logger;
const MAX_HEAP_SIZE: u64 = 1024 * 8192;
const MAX_STACK_SIZE: usize = 1024 * 64;
#[repr(C, align(4))]
struct Stack([u8; MAX_STACK_SIZE]);
static STACK: Stack = Stack([0; MAX_STACK_SIZE]);

static FIVE_LEVEL_PAGING_REQ: Limine5LevelPagingRequest = Limine5LevelPagingRequest::new(0);
static HHDM_REQ: LimineHhdmRequest = LimineHhdmRequest::new(0);
static MEM_MAP_REQ: LimineMemmapRequest = LimineMemmapRequest::new(0);
static RSDP_REQ: LimineRsdpRequest = LimineRsdpRequest::new(0);
static SMP_REQ: LimineSmpRequest = LimineSmpRequest::new(0);
static STACK_SIZE_REQ: LimineStackSizeRequest = LimineStackSizeRequest::new(0).stack_size(MAX_STACK_SIZE as u64);

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
pub extern "C" fn _start() -> ! {
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
let five_level_paging_resp = FIVE_LEVEL_PAGING_REQ.get_response().get();

if let Some(_) = five_level_paging_resp {
info!("Using 5-level paging");
warn!("Five-level paging is not yet fully supported");
warn!("Expect memory errors and other weird behavior");
} else {
info!("Using 4-level paging");
}
    info!("Initializing interrupt subsystem");
    libk::interrupts::init_idt();
    libk::gdt::init();
    info!("Initializing memory manager");
    let mmap = MEM_MAP_REQ.get_response().get().expect("Bootloader did not provide a memory map!");
    let rsdp = RSDP_REQ.get_response().get().expect("Bootloader did not provide an RSDP address!");
    libk::memory::init_memory_map(memmap.memmap(), rsdp);
    let mut idx = usize::MAX;
    let addrs = loop {
        idx = idx.wrapping_add(1);
        let entry = memmap.memmap().iter().nth(idx);
        let entry = *entry;
        if entry.type != LimineMemoryMapEntryType::Usable {
            continue;
        }
        if entry.len > MAX_HEAP_SIZE {
            let base = entry.base;
            break (base, base + MAX_HEAP_SIZE);
        }
    };
    if addrs == (0, 0) {
        panic!("Can't find a memory region for the heap!");
    }
let hhdm_resp = HHDM_REQ.get_response().get();
    libk::memory::init(hhdm_resp.offset, addrs.0, MAX_HEAP_SIZE);
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
