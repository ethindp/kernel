// SPDX-License-Identifier: MPL-2.0
#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(option_result_contains)]
#![feature(type_alias_impl_trait)]
#![feature(alloc_layout_extra)]
#![feature(const_in_array_repeat_expressions)]
#![feature(llvm_asm)]
#![feature(wake_trait)]
#![allow(dead_code)]
#![forbid(
    warnings,
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
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    variant_size_differences
)]
#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    box_pointers
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
/// The vga module contains functions for interacting with the VGA buffer.
pub mod vga;
use bit_field::BitField;
use linked_list_allocator as _;
use part as _;

/// Initializes the kernel and sets up required functionality.
pub fn init() {
    use task::cooperative::executor::Executor;
    use task::AsyncTask;
    let mut executor = Executor::new();
    executor.spawn(AsyncTask::new(gdt::init()));
    executor.spawn(AsyncTask::new(interrupts::init_stage2()));
    executor.spawn(AsyncTask::new(rtc::init()));
    executor.spawn(AsyncTask::new(pci::init()));
    executor.spawn(AsyncTask::new(init_nvme()));
    executor.run();
}

/// This function is designed as a failsafe against memory corruption if we panic.
pub fn idle_forever() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

async fn init_nvme() {
    loop {
        if pci::find_device(0x01, 0x08, 0x02).await.is_some() {
            break;
        }
        x86_64::instructions::hlt();
    }
    let mut int = 32u8;
    {
        let dev = pci::find_device(0x01, 0x08, 0x02).await.unwrap();
        let mut cmd = pci::read_word(dev.phys_addr as usize, pci::COMMAND);
        cmd.set_bit(10, false);
        cmd.set_bit(2, true);
        cmd.set_bit(1, true);
        pci::write_word(dev.phys_addr as usize, pci::COMMAND, cmd);
        int += pci::read_byte(dev.phys_addr as usize, pci::INT_LINE);
    }
    let bars = [
        pci::get_bar(0, 0x01, 0x08, 0x02).await.unwrap() as u64,
        pci::get_bar(1, 0x01, 0x08, 0x02).await.unwrap() as u64,
        pci::get_bar(2, 0x01, 0x08, 0x02).await.unwrap() as u64,
        pci::get_bar(3, 0x01, 0x08, 0x02).await.unwrap() as u64,
        pci::get_bar(4, 0x01, 0x08, 0x02).await.unwrap() as u64,
        pci::get_bar(5, 0x01, 0x08, 0x02).await.unwrap() as u64,
    ];
    let controller = unsafe {
        nvme::NvMeController::new(
            bars,
            int,
            |start, size| crate::memory::allocate_phys_range(start, start + size),
            |start, size| crate::memory::free_range(start, start + size),
            interrupts::register_interrupt_handler,
        )
    };
    controller.init().await;
}
