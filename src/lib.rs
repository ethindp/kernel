#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(slice_from_raw_parts)]
#![allow(dead_code)]
/// The drivers module contains drivers for various hardware devices.
pub mod drivers;
/// The gdt module contains basic GDT functionality.
/// When initialized, a separate stack is set up for the kernel to run in to ensure that the original is not comprimised.
pub mod gdt;
/// The interrupts module contains functions to set up the IDT.
/// It also utilizes full AIO support for keyboards and other devices.
pub mod interrupts;
/// The memory module contains functions for managing memory.
pub mod memory;
/// The pci module contains functions for reading from PCI devices and enumerating PCI buses via the "brute-force" method.
/// As we add drivers that require the PCI buss in, the ::probe() function of this module will be extended to load those drivers when the probe is in progress. This will then create a "brute-force and configure" method.
pub mod pci;
/// The registers module contains functions for retrieving all CPU registers.
// It is only used for debugging and kernel information purposes.
pub mod registers;
// Te smbios module contains SMBIOS functions
// pub mod smbios;
/// The tasking module contains multitasking-related functions
pub mod tasking;
/// The vga module contains functions for interacting with the VGA buffer.
pub mod vga;
use cpuio::{inb, outb};

/// Initializes the kernel and sets up required functionality.
pub fn init() {
    printkln!("Loading GDT");
    gdt::init();
    printkln!("Loading IDT");
    interrupts::initialize_idt();
    printkln!("Initializing chained PICs");
    unsafe { interrupts::PICS.lock().initialize() };
    printkln!("Enabling interrupts");
    x86_64::instructions::interrupts::enable();
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
    //smbios::init();
    // Now, probe the PCI bus.
    pci::init();
    // Request other drivers to initialize
    drivers::hid::keyboard::init();
    drivers::sound::hda::init();
    drivers::storage::ahci::init();
}

/// This function is designed as a failsafe against memory corruption if we panic.
pub fn idle_forever() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
