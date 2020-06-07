#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(option_result_contains)]
#![feature(const_if_match)]
#![feature(type_alias_impl_trait)]
#![feature(alloc_layout_extra)]
#![feature(const_in_array_repeat_expressions)]
#![feature(llvm_asm)]
#![allow(dead_code)]
#![deny(
    array_into_iter,
    bare_trait_objects,
    deprecated,
    ellipsis_inclusive_range_patterns,
    exported_private_dependencies,
    illegal_floating_point_literal_pattern,
    improper_ctypes,
    incomplete_features,
    intra_doc_link_resolution_failure,
    invalid_value,
    irrefutable_let_patterns,
    late_bound_lifetime_arguments,
    mutable_borrow_reservation_conflict,
    non_shorthand_field_patterns,
    non_snake_case,
    non_upper_case_globals,
    no_mangle_generic_items,
    overlapping_patterns,
    path_statements,
    private_in_public,
    proc_macro_derive_resolution_fallback,
    redundant_semicolons,
    renamed_and_removed_lints,
    safe_packed_borrows,
    stable_features,
    trivial_bounds,
    type_alias_bounds,
    tyvar_behind_raw_pointer,
    unconditional_recursion,
    unknown_lints,
    unnameable_test_items,
    unreachable_code,
    unreachable_patterns,
    unstable_name_collisions,
    unused_allocation,
    unused_assignments,
    unused_attributes,
    unused_comparisons,
    unused_doc_comments,
    unused_features,
    unused_imports,
    unused_labels,
    unused_macros,
    unused_must_use,
    unused_mut,
    unused_parens,
    unused_unsafe,
    unused_variables,
    where_clauses_object_safety,
    while_true,
    ambiguous_associated_items,
    arithmetic_overflow,
    const_err,
    ill_formed_attribute_input,
    invalid_type_param_default,
    macro_expanded_macro_exports_accessed_by_absolute_paths,
    missing_fragment_specifier,
    mutable_transmutes,
    no_mangle_const_items,
    order_dependent_trait_objects,
    overflowing_literals,
    patterns_in_fns_without_body,
    pub_use_of_private_extern_crate,
    soft_unstable,
    unknown_crate_types
)]
#![deny(clippy::all)]
extern crate alloc;
/// The disk module contains bear-bones code to support reading from ATA disk drives.
pub mod disk;
/// The 
/// The gdt module contains basic GDT functionality.
/// When initialized, a separate stack is set up for the kernel to run in to ensure that the original is not compromised when double faults occur.
pub mod gdt;
/// The interrupts module contains functions to set up the IDT.
/// It also utilizes full AIO support for keyboards and other devices.
pub mod interrupts;
/// The memory module contains functions for managing memory.
pub mod memory;
/// The pci module contains functions for reading from PCI devices and enumerating PCI buses via the "brute-force" method.
/// As we add drivers that require the PCI buss in, the ::probe() function of this module will be extended to load those drivers when the probe is in progress. This will then create a "brute-force and configure" method.
pub mod pci;
/// The vga module contains functions for interacting with the VGA buffer.
pub mod vga;
use cpuio::{inb, outb};

/// Initializes the kernel and sets up required functionality.
pub fn init() {
    printkln!("Enabling interrupts, second stage");
    gdt::init();
    interrupts::init_stage2();
    printkln!("Configuring RTC");
    // There's a very high chance we'll immediately get interrupts fired. We turn them off here to prevent crashes while we set up the RTC.
    x86_64::instructions::interrupts::without_interrupts(|| {
        // Enable the real time clock
        // We must be careful because if we mess this up, we could leave the RTC in an
        // undefined state. Unlike the PIC timer/PIT, this will survive cold reboots and boots.
        let rate = 3 & 0x0F;
        unsafe {
            // Control register A of the RTC and temporarily disable NMIs
            outb(0x8A, 0x70);
            // Read initial value of register A
            let mut prev = inb(0x71);
            // Reset index to register A
            outb(0x8A, 0x70);
            // Right tick freq to register A
            outb((prev & 0xF0) | rate, 0x71);
            // Switch to register B
            outb(0x8B, 0x70);
            // Read current value of register B
            prev = inb(0x71);
            // Re-control register B.
            outb(0x8B, 0x70);
            // Enable RTC
            outb(prev | 0x40, 0x71);
        }
    });
    // Now, probe the PCI bus.
    pci::init();
}

/// This function is designed as a failsafe against memory corruption if we panic.
pub fn idle_forever() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
