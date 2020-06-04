use crate::{drivers::hid::keyboard::*, gdt};
use cpuio::{inb, outb};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use raw_cpuid::*;
use spin::{Mutex, RwLock};
use x86_64::registers::model_specific::Msr;
use x86_64::{
    structures::idt::PageFaultErrorCode,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

/// This enumeration contains a list of all IRQs.
#[repr(u8)]
pub enum InterruptType {
    Timer = 32, // IRQ 0 - system timer (cannot be changed)
    Keyboard,   // IRQ 1 - keyboard controller (cannot be changed)
    Cascade, // IRQ 2 - cascaded signals from IRQs 8-15 (any devices configured to use IRQ 2 will actually be using IRQ 9)
    Uart1, // IRQ 3 - serial port controller for serial port 2 (shared with serial port 4, if present)
    Serial1, // IRQ 4 - serial port controller for serial port 1 (shared with serial port 3, if present)
    Parallel, // IRQ 5 - parallel port 2 and 3  or  sound card
    Floppy,  // IRQ 6 - floppy disk controller
    Lpt1, // IRQ 7 - parallel port 1. It is used for printers or for any parallel port if a printer is not present. It can also be potentially be shared with a secondary sound card with careful management of the port.
    Rtc,  // IRQ 8 - real-time clock (RTC)
    Acpi, // IRQ 9 - Advanced Configuration and Power Interface (ACPI) system control interrupt on Intel chipsets.
    // Other chipset manufacturers might use another interrupt for this purpose, or make it available for the use of peripherals (any devices configured to use IRQ 2 will actually be using IRQ 9)
    Open1, // IRQ 10 - The Interrupt is left open for the use of peripherals (open interrupt/available, SCSI or NIC)
    Open2, // IRQ 11 - The Interrupt is left open for the use of peripherals (open interrupt/available, SCSI or NIC)
    Mouse, // IRQ 12 - mouse on PS/2 connector
    Coprocessor, // IRQ 13 - CPU co-processor  or  integrated floating point unit  or  inter-processor interrupt (use depends on OS)
    PrimaryAta, // IRQ 14 - primary ATA channel (ATA interface usually serves hard disk drives and CD drives)
    SecondaryAta, // IRQ 15 - secondary ATA channel
}

impl InterruptType {
    fn convert_to_u8(self) -> u8 {
        self as u8
    }

    fn convert_to_usize(self) -> usize {
        usize::from(self.convert_to_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
// Handle BPs
        idt.breakpoint.set_handler_fn(handle_bp);
// Handle DFs (on our set-up separate 4K kernel stack)
        unsafe {
            idt.double_fault
                .set_handler_fn(handle_df)
                .set_stack_index(gdt::DF_IST_IDX);
        }
        idt.page_fault.set_handler_fn(handle_pf);
        idt.overflow.set_handler_fn(handle_of);
        idt.bound_range_exceeded.set_handler_fn(handle_br);
        idt.invalid_opcode.set_handler_fn(handle_ud);
        idt.device_not_available.set_handler_fn(handle_nm);
        idt.general_protection_fault.set_handler_fn(handle_gp);
        idt[InterruptType::Keyboard.convert_to_usize()].set_handler_fn(handle_keyboard);
        idt[InterruptType::Rtc.convert_to_usize()].set_handler_fn(handle_rtc);
        idt[InterruptType::Timer.convert_to_usize()].set_handler_fn(handle_timer);
        idt
    };
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            layouts::Us104Key,
            ScancodeSet1,
            HandleControl::MapLettersToUnicode
        ));
    static ref TICK_COUNT: RwLock<u128> = RwLock::new(0u128);
    static ref KEY_CMD: Mutex<u8> = Mutex::new(0u8);
}

pub fn init_stage1() {
    use crate::printkln;
    printkln!("INTR: Stage 1 initialization started");
    unsafe {
        printkln!("INTR: PIC: Acquiring masks");
        let saved_mask1 = inb(0x21);
        let saved_mask2 = inb(0xA1);
        printkln!("INTR: PIC: Masks: {:X}h, {:X}h", saved_mask1, saved_mask2);
        printkln!("INTR: PIC: Sending initialization command");
        outb(0x11, 0x20);
        outb(0, 0x80);
        outb(0x11, 0xA0);
        outb(0, 0x80);
        printkln!("INTR: PIC: Setting base offsets to 20h and 28h");
        outb(0x20, 0x21);
        outb(0, 0x80);
        outb(0x28, 0xA1);
        outb(0, 0x80);
        printkln!("INTR: PIC: Setting up chain for master and slave");
        outb(0x04, 0x21);
        outb(0, 0x80);
        outb(0x02, 0xA1);
        outb(0, 0x80);
        printkln!("INTR: PIC: Setting mode to 1h");
        outb(0x01, 0x21);
        outb(0, 0x80);
        outb(0x01, 0xA1);
        outb(0, 0x80);
        printkln!("INTR: PIC: Restoring PIC masks");
        outb(saved_mask1, 0x21);
        outb(0, 0x80);
        outb(saved_mask2, 0xA1);
        outb(0, 0x80);
    }
    printkln!("INTR: Loading IDT");
    IDT.load();
    printkln!("INTR: Enabling interrupts");
    x86_64::instructions::interrupts::enable();
    printkln!("INTR: Stage 1 initialization complete");
}

pub fn init_stage2() {
    use crate::memory::{allocate_phys_range, read_dword, write_dword};
    use crate::printkln;
    x86_64::instructions::interrupts::disable();
    printkln!("INTR: loading IDT");
    IDT.load();
    if is_apic_available() {
        allocate_phys_range(apic_addr(), apic_addr() + 0x530);
        // Initialize PIC, then mask everything
        printkln!("INTR: APIC is available and usable; configuring");
        unsafe {
            let saved_mask1 = inb(0x21);
            let saved_mask2 = inb(0xA1);
            outb(0x11, 0x20);
            outb(0, 0x80);
            outb(0x11, 0xA0);
            outb(0, 0x80);
            outb(0x20, 0x21);
            outb(0, 0x80);
            outb(0x28, 0xA1);
            outb(0, 0x80);
            outb(0x04, 0x21);
            outb(0, 0x80);
            outb(0x02, 0xA1);
            outb(0, 0x80);
            outb(0x01, 0x21);
            outb(0, 0x80);
            outb(0x01, 0xA1);
            outb(0, 0x80);
            outb(saved_mask1, 0x21);
            outb(0, 0x80);
            outb(saved_mask2, 0xA1);
            outb(0, 0x80);
            outb(0xFF, 0xA1);
            outb(0, 0x80);
            outb(0xFF, 0x21);
            outb(0, 0x80);
        }
        write_dword(apic_addr() + 0xF0, read_dword(apic_addr() + 0xF0) | 0x100);
        printkln!("INTR: APIC configuration complete");
    } else {
        printkln!("INTR: APIC not available/supported, falling back to 8259 PIC");
        printkln!("INTR: Configuring PIC");
        unsafe {
            let saved_mask1 = inb(0x21);
            let saved_mask2 = inb(0xA1);
            outb(0x11, 0x20);
            outb(0, 0x80);
            outb(0x11, 0xA0);
            outb(0, 0x80);
            outb(0x20, 0x21);
            outb(0, 0x80);
            outb(0x28, 0xA1);
            outb(0, 0x80);
            outb(0x04, 0x21);
            outb(0, 0x80);
            outb(0x02, 0xA1);
            outb(0, 0x80);
            outb(0x01, 0x21);
            outb(0, 0x80);
            outb(0x01, 0xA1);
            outb(0, 0x80);
            outb(saved_mask1, 0x21);
            outb(0, 0x80);
            outb(saved_mask2, 0xA1);
            outb(0, 0x80);
        }
        printkln!("INTR: PIC configuration complete");
    }
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn handle_bp(stack_frame: &mut InterruptStackFrame) {
    // All we do here is notify the user and continue on.
    use crate::printkln;
    printkln!(
        "Hardware breakpoint interrupt received:\n{:#?}",
        stack_frame
    );
}

extern "x86-interrupt" fn handle_df(stack_frame: &mut InterruptStackFrame, error_code: u64) -> ! {
    unsafe {
        llvm_asm!("push rax" :::: "intel");
    }
    panic!(
        "EXCEPTION: DOUBLE FAULT({})\n{:#?}",
        error_code, stack_frame,
    );
}

extern "x86-interrupt" fn handle_timer(_: &mut InterruptStackFrame) {
    // Acquire the keyboard command spinlock.
    let mut cmd = KEY_CMD.lock();
    unsafe {
        // Figure out what we've got
        if let Some(command) = dequeue_command() {
            // Keyboard command, output it to the keyboard and set the keyboard command static reference to this command for later use.
            *cmd = command;
            outb(command, 0x60);
        }
    }
    signal_eoi(InterruptType::Timer.convert_to_u8());
}

extern "x86-interrupt" fn handle_rtc(_stack_frame: &mut InterruptStackFrame) {
    if let Some(mut tc) = TICK_COUNT.try_write() {
        *tc += 1;
    }
    signal_eoi(InterruptType::Rtc.convert_to_u8());
    unsafe {
        outb(0x0C, 0x70);
        inb(0x71);
    }
}

extern "x86-interrupt" fn handle_keyboard(_stack_frame: &mut InterruptStackFrame) {
    let reply = unsafe { inb(0x60) };
    let mut keyboard = KEYBOARD.lock();
    let cmd = KEY_CMD.lock();
    match reply {
        // Key error / buffer overrun
        0x00 | 0xFF => notify_key_error(),
        // Self-test completed
        0xAA => notify_self_test_succeeded(),
        // Acknowledgement
        0xFA => {
            notify_ack(*cmd);
            if *cmd == 0xF2 {
                // Keyboard is sending ID bytes
                let byte1 = unsafe { inb(0x60) };
                let byte2 = unsafe { inb(0x60) };
                notify_id_finished(byte1, byte2);
            }
        }
        // Resend requested
        0xFE => notify_resend(*cmd),
        // Self-test failed
        0xFC | 0xFD => notify_self_test_failed(),
        // Another key, pass it on.
        key => {
            if let Ok(Some(key_event)) = keyboard.add_byte(key) {
                if let Some(keycode) = keyboard.process_keyevent(key_event) {
                    match keycode {
                        DecodedKey::Unicode(byte) => {
                            notify_key((Some(byte), None));
                            keyboard.clear();
                        }
                        DecodedKey::RawKey(raw) => {
                            notify_key((None, Some(raw)));
                            keyboard.clear();
                        }
                    }
                }
            }
        }
    }
    signal_eoi(InterruptType::Keyboard.convert_to_u8());
}

extern "x86-interrupt" fn handle_pf(_: &mut InterruptStackFrame, error_code: PageFaultErrorCode) {
    unsafe {
        llvm_asm!("push rax" :::: "intel");
    }
    use crate::idle_forever;
    use crate::printkln;
    use x86_64::registers::control::Cr2;
    let addr = Cr2::read();
    let ec = error_code.bits();
    printkln!(
        "Page fault: {} while {} to memory address {:X}h",
        if (ec & 1) > 0 {
            "Protection violation"
        } else if !(ec & 1) > 0 {
            "Page not present"
        } else if (ec & 1 << 2) > 0 {
            "Possible privilege violation (user mode)"
        } else if !(ec & 1 << 2) > 0 {
            "Possible privilege violation (kernel mode)"
        } else if ec & 1 << 3 > 0 {
            "Attempted read of reserved PTT entry"
        } else if ec & 1 << 4 > 0 {
            "Instruction fetch"
        } else {
            "unknown cause"
        },
        if ec & 1 << 1 > 0 {
            "writing"
        } else {
            "reading"
        },
        addr.as_u64()
    );
    idle_forever();
}

extern "x86-interrupt" fn handle_of(_: &mut InterruptStackFrame) {
    use crate::printkln;
    printkln!("Warning: can't execute calculation: overflow");
}

extern "x86-interrupt" fn handle_br(stack: &mut InterruptStackFrame) {
    panic!(
        "Cannot continue: bounds range exceeded.\nStack:\n{:?}",
        stack,
    );
}

extern "x86-interrupt" fn handle_ud(stack: &mut InterruptStackFrame) {
    panic!("Cannot continue: invalid opcode!\nStack:\n{:?}", stack,);
}

extern "x86-interrupt" fn handle_nm(stack: &mut InterruptStackFrame) {
    panic!("Can't continue: device unavailable!\nStack:\n{:?}", stack,);
}

extern "x86-interrupt" fn handle_gp(_: &mut InterruptStackFrame, ec: u64) {
    unsafe {
        llvm_asm!("push rax" :::: "intel");
    }
    use crate::printkln;
    printkln!("Cannot continue: protection violation, error code {}", ec,);
}

fn is_apic_available() -> bool {
    use bit_field::BitField;
    let apic_available_in_msr = {
        let apicbase = Msr::new(0x1B);
        unsafe { apicbase.read().get_bit(11) }
    };
    apic_available_in_msr && cpuid!(1).ecx.get_bit(9)
}

fn apic_addr() -> u64 {
    use bit_field::BitField;
    let apicbase = Msr::new(0x1B);
    unsafe { apicbase.read().get_bits(12..52) }
}

fn signal_eoi(interrupt: u8) {
    if !is_apic_available() {
        if 32 <= interrupt && interrupt < 32 + 8 {
            unsafe {
                outb(0x20, 0x20);
            }
        } else if 40 <= interrupt && interrupt < 40 + 8 {
            unsafe {
                outb(0x20, 0xA0);
            }
        } else {
            unsafe {
                outb(0x20, 0x20);
                outb(0x20, 0xA0);
            }
        }
    } else {
        use crate::memory::write_dword;
        write_dword(apic_addr() + 0xB0, 0);
    }
}

/// Gets the tick count that has passed since we started counting (since the RTC was set up).
/// Can handle up to 340282366920938463463374607431768211456 ticks,
/// Which, if is in microseconds, is about 10 septillion 780 sextillion years
pub fn get_tick_count() -> u128 {
    *TICK_COUNT.read()
}

/// Sleeps for the given duration of microseconds.
pub fn sleep_for(duration: u128) {
    let mut count = get_tick_count();
    let end = count + duration;
    while count != end {
        count = get_tick_count();
        x86_64::instructions::hlt();
    }
}
