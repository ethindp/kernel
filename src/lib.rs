// SPDX-License-Identifier: MPL-2.0
#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(option_result_contains)]
#![feature(type_alias_impl_trait)]
#![feature(alloc_layout_extra)]
#![feature(const_in_array_repeat_expressions)]
#![feature(llvm_asm)]
#![allow(dead_code)]
#![forbid(warnings,
absolute_paths_not_starting_with_crate,
anonymous_parameters,
box_pointers,
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
unused_extern_crates,
unused_import_braces,
unused_lifetimes,
variant_size_differences
)]
#![deny(
missing_copy_implementations,
missing_debug_implementations,
unused_results
)]
#![forbid(clippy::all)]
extern crate alloc;
// The acpi module contains acpi initialization routines
pub mod acpi;
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
use linked_list_allocator as _;
use part as _;


/// Initializes the kernel and sets up required functionality.
pub fn init() {
    printkln!("init: initializing GDT");
    gdt::init();
    printkln!("init: Enabling interrupts, second stage");
    interrupts::init_stage2();
    printkln!("init: configuring RTC");
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
    pci::init();
}

/// This function is designed as a failsafe against memory corruption if we panic.
pub fn idle_forever() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
