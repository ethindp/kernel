//! The libk crate contains core kernel code.
//! This crate is suitable for inclusion in kernel drivers.

#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(option_result_contains)]
#![feature(alloc_layout_extra)]
#![feature(async_closure)]
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
    missing_abi,
    missing_docs,
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
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    variant_size_differences,
    trivial_casts
)]
#![deny(missing_copy_implementations, missing_debug_implementations, warnings)]
#![forbid(clippy::all)]
extern crate alloc;
/// The acpi module contains acpi initialization routines
pub mod acpi;
/// The gdt module contains basic GDT functionality.
pub mod gdt;
/// The interrupts module contains functions to set up the IDT.
/// It also utilizes full AIO support for keyboards and other devices.
pub mod interrupts;
/// The memory module contains functions for managing memory.
pub mod memory;
/// The pci module contains functions for reading from PCI devices and enumerating PCI buses.
pub mod pci;
/// The rtc modue/le contains RTC initialization code
pub mod rtc;
/// The task module controls cooperative and preemptive multitasking schedulers. The
/// cooperative scheduler runs in the kernel while the preemptive scheduler will run in
/// userspace once implemented.
#[allow(missing_debug_implementations, missing_copy_implementations)]
pub mod task;
/// The timer module contains delaying and sleeping functionality
pub mod timer;

/// Initializes the kernel and sets up required functionality.
#[cold]
pub fn init() {
    use core::any::TypeId;
    use log::info;
    use task::cooperative::executor::Executor;
    use task::AsyncTask;
    info!(
        "Detected endienness is {}",
        if TypeId::of::<byteorder::NativeEndian>() == TypeId::of::<byteorder::LittleEndian>() {
            "little endien"
        } else {
            "big endien"
        }
    );
    let mut executor = Executor::new();
    executor.spawn(AsyncTask::new(acpi::init()));
    executor.spawn(AsyncTask::new(pci::init()));
    executor.spawn(AsyncTask::new(rtc::init()));
    executor.run();
}

/// This function is designed as a failsafe against memory corruption if we panic or suffer a fatal error.
pub fn idle_forever() -> ! {
    use core::arch::x86_64::_mm_pause;
    loop {
        unsafe {
            _mm_pause();
        }
    }
}
