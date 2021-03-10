//! The libk crate contains core kernel code.
//! This crate is suitable for inclusion in kernel drivers.

#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(option_result_contains)]
#![feature(type_alias_impl_trait)]
#![feature(alloc_layout_extra)]
#![feature(llvm_asm)]
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
    missing_docs,
    non_ascii_idents,
    pointer_structural_match,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unaligned_references,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    variant_size_differences,
    ambiguous_associated_items,
    arithmetic_overflow,
    conflicting_repr_hints,
    const_err,
    ill_formed_attribute_input,
    incomplete_include,
    invalid_type_param_default,
    macro_expanded_macro_exports_accessed_by_absolute_paths,
    mutable_transmutes,
    no_mangle_const_items,
    order_dependent_trait_objects,
    overflowing_literals,
    patterns_in_fns_without_body,
    pub_use_of_private_extern_crate,
    soft_unstable,
    unconditional_panic,
    unknown_crate_types,
    useless_deprecated
)]
#![deny(warnings, missing_copy_implementations, missing_debug_implementations)]
#![forbid(clippy::all)]
extern crate alloc;
/// The acpi module contains acpi initialization routines
pub mod acpi;
/// The disk module defines a trait and various enumerations for disk implementations.
pub mod disk;
/// The fs module contains support modules for various built-in file systems.
pub mod fs;
/// The gdt module contains basic GDT functionality.
/// When initialized, a separate stack is set up for the kernel to run in to ensure that the
///original is not compromised when double faults occur.
pub mod gdt;
/// The interrupts module contains functions to set up the IDT.
/// It also utilizes full AIO support for keyboards and other devices.
pub mod interrupts;
/// The memory module contains functions for managing memory.
pub mod memory;
/// The nvme module contains core NvMe support required for future higher-level bootstrapping.
///
/// ## Commands
///
/// At the core of NVMe communication is the command. A command is submitted in queues, one for submission and one for completion. The NvMe driver handles the creation and deletion of queues.
///
/// There are two types of queues: an admin and an I/O queue.
///
/// * An admin queue allows administration of an NVMe controller, function, or subsystem. There is only one admin queue pair per controller. The admin queue may have up to 65536 entries.
/// * An I/O queue allows the submission of NvM I/O commands. There may be up to 65534 I/O queues. Each queue may have up to 65536 entries.
///
/// Though the above limits are the absolute maximums, it is unlikely that most controllers will support them. Rather, it is more likely that a controller will support a much lower hard limit of entries in a queue, e.g.: up to 2048 entries per queue, rather than supporting 65536 entries. The higher the maximum entry limit is, the more memory that is consumed. In embedded environemnts or on systems with low amounts of memory, it may be desirable to severely limit the maximum number of queue entries to allow more memory for other tasks.
///
/// The maximum entries limit allows for a large degree of parallelism. The NVMe specification allows many applications to submit many commands simultaneously, and then have the controller process all the entries at once. To achieve an even higher level of parallelism, one can have multiple I/O queues. For example, an operating system may have one queue per process priority level such that each process will, with a high degree of certainty, submit a command in an empty queue. To facilitate such systems, NVMe contains a priority mechanism whereby a freshly created queue can tell the controller precisely what priority to allocate to each queue. This is called the arbitration mechanism. The specification defines only one, the round-robin arbitration mechanism, however it is possible that vendors may define others. In the round-robin arbitration system, the highest priority queues are processed first, followed by lower priority ones.
pub mod nvme;
/// The pci module contains functions for reading from PCI devices and enumerating PCI buses
/// via the "brute-force" method.
/// As we add drivers that require the PCI buss in, the ::probe() function of this module
/// will be extended to load those drivers when the probe is in progress. This will then
/// create a "brute-force and configure" method.
pub mod pci;
/// The rtc module contains RTC initialization code
pub mod rtc;
/// The task module controls cooperative and preemptive multitasking schedulers. The
/// cooperative scheduler runs in the kernel while the preemptive scheduler will run in
/// userspace once implemented.
#[allow(
    missing_debug_implementations,
    missing_copy_implementations,
    box_pointers
)]
pub mod task;
use block_device as _;
use linked_list_allocator as _;
use zerocopy as _;

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
    executor.spawn(AsyncTask::new(gdt::init()));
    executor.spawn(AsyncTask::new(interrupts::init_stage2()));
    executor.spawn(AsyncTask::new(pci::init()));
    executor.spawn(AsyncTask::new(rtc::init()));
    executor.run();
}

/// This function is designed as a failsafe against memory corruption if we panic.
pub fn idle_forever() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
