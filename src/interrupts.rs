use crate::drivers::hid::keyboard::*;
use crate::gdt;
use crate::registers;
use cpuio::{inb, outb};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259_simple::ChainedPics;
use spin;
use spin::Mutex;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(32, 32 + 8) });

/// This enumeration contains a list of all IRQs.
#[repr(u8)]
pub enum InterruptType {
    Timer = 32,   // IRQ 0 - system timer (cannot be changed)
    Keyboard,     // IRQ 1 - keyboard controller (cannot be changed)
    Cascade, // IRQ 2 - cascaded signals from IRQs 8-15 (any devices configured to use IRQ 2 will actually be using IRQ 9)
    Uart1, // IRQ 3 - serial port controller for serial port 2 (shared with serial port 4, if present)
    Serial1, // IRQ 4 - serial port controller for serial port 1 (shared with serial port 3, if present)
    Parallel, // IRQ 5 - parallel port 2 and 3  or  sound card
    Floppy,  // IRQ 6 - floppy disk controller
    Lpt1, // IRQ 7 - parallel port 1. It is used for printers or for any parallel port if a printer is not present. It can also be potentially be shared with a secondary sound card with careful management of the port.
    Rtc,  // IRQ 8 - real-time clock (RTC)
    Acpi, // IRQ 9 - Advanced Configuration and Power Interface (ACPI) system control interrupt on Intel chipsets. Other chipset manufacturers might use another interrupt for this purpose, or make it available for the use of peripherals (any devices configured to use IRQ 2 will actually be using IRQ 9)
    Open1, // IRQ 10 - The Interrupt is left open for the use of peripherals (open interrupt/available, SCSI or NIC)
    Open2, // IRQ 11 - The Interrupt is left open for the use of peripherals (open interrupt/available, SCSI or NIC)
    Mouse, // IRQ 12 - mouse on PS/2 connector
    Coprocessor, // IRQ 13 - CPU co-processor  or  integrated floating point unit  or  inter-processor interrupt (use depends on OS)
    PrimaryAta, // IRQ 14 - primary ATA channel (ATA interface usually serves hard disk drives and CD drives)
    SecondaryAta, // IRQ 15 - secondary ATA channel
}

impl InterruptType {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
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
        idt[InterruptType::Keyboard.as_usize()].set_handler_fn(handle_keyboard);
        idt[InterruptType::Rtc.as_usize()].set_handler_fn(handle_rtc);
        idt[InterruptType::Timer.as_usize()].set_handler_fn(handle_timer);
        idt
    };
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            layouts::Us104Key,
            ScancodeSet1,
            HandleControl::MapLettersToUnicode
        ));
    static ref TICK_COUNT: Mutex<u128> = Mutex::new(0u128);
    static ref KEY_CMD: Mutex<u8> = Mutex::new(0u8);
}

pub fn initialize_idt() {
    IDT.load();
}

extern "x86-interrupt" fn handle_bp(stack_frame: &mut InterruptStackFrame) {
    // All we do here is notify the user and continue on.
    use crate::printkln;
    printkln!(
        "Hardware breakpoint interrupt received:\n{:#?}",
        stack_frame
    );
}

extern "x86-interrupt" fn handle_df(stack_frame: &mut InterruptStackFrame, error_code: u64) {
unsafe {asm!("push rax" :::: "intel");} 
    panic!(
        "EXCEPTION: DOUBLE FAULT({})\n{:#?}\n{}",
        error_code,
        stack_frame,
        registers::CPURegs::read()
    );
}

extern "x86-interrupt" fn handle_timer(_: &mut InterruptStackFrame) {
    // Acquire the keyboard command spinlock.
    let mut cmd = KEY_CMD.lock();
    unsafe {
        // Figure out what we've got
        match dequeue_command() {
            Some(command) => {
                // Keyboard command, output it to the keyboard and set the keyboard command static reference to this command for later use.
                *cmd = command;
                outb(command, 0x60);
            }
            None => (),
        }
        PICS.lock()
            .notify_end_of_interrupt(InterruptType::Timer.as_u8());
    }
}

extern "x86-interrupt" fn handle_rtc(_stack_frame: &mut InterruptStackFrame) {
    unsafe {
        let mut tc = TICK_COUNT.lock();
        *tc += 1;
        PICS.lock()
            .notify_end_of_interrupt(InterruptType::Rtc.as_u8());
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
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptType::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn handle_pf(
    stack: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
unsafe {asm!("push rax" :::: "intel");} 
    use crate::idle_forever;
    use crate::{printk, printkln};
    use x86_64::registers::control::Cr2;
    use bit_field::BitField;
    use crate::memory::allocate_page_range;
    let addr = Cr2::read();
    let ec = error_code.bits();
            printk!("Page fault: ");
    if (ec & 1<<0) > 0 {
    printkln!("Protection violation");
        } else if !(ec & 1 << 0) > 0 {
        printkln!("Page not present");
    } else if (ec & 1<<2) > 0 {
    printkln!("Possible privilege violation (user mode)");
    } else if !(ec & 1<<2) > 0 {
    printkln!("Possible privilege violation (kernel mode)");
    } else if ec & 1 << 3 > 0 {
    printkln!("Attempted read of reserved PTT entry");
    } else if ec & 1 << 4 > 0 {
    printkln!("Instruction fetch");
    }
    if ec & 1 << 1 > 0 {
        printkln!("Possibly caused by write to memory address {:X}h", addr.as_u64());
    } else {
        printkln!("Possibly caused by read from memory address {:X}h", addr.as_u64());
    }
    idle_forever();
}

extern "x86-interrupt" fn handle_of(_: &mut InterruptStackFrame) {
    use crate::printkln;
    printkln!("Warning: can't execute calculation: overflow");
}

extern "x86-interrupt" fn handle_br(stack: &mut InterruptStackFrame) {
    panic!(
        "Cannot continue: bounds range exceeded.\nStack:\n{:?}\n{}",
        stack,
        registers::CPURegs::read()
    );
}

extern "x86-interrupt" fn handle_ud(stack: &mut InterruptStackFrame) {
    panic!(
        "Cannot continue: invalid opcode!\nStack:\n{:?}\n{}",
        stack,
        registers::CPURegs::read()
    );
}

extern "x86-interrupt" fn handle_nm(stack: &mut InterruptStackFrame) {
    panic!(
        "Can't continue: device unavailable!\nStack:\n{:?}\n{}",
        stack,
        registers::CPURegs::read()
    );
}

extern "x86-interrupt" fn handle_gp(_: &mut InterruptStackFrame, ec: u64) {
unsafe {asm!("push rax" :::: "intel");} 
    use crate::printkln;
    printkln!(
        "Cannot continue: protection violation, error code {}\n{}",
        ec,
        registers::CPURegs::read()
    );
}

/// Gets the tick count that has passed since we started counting (since the RTC was set up).
/// Can handle up to 340282366920938463463374607431768211456 ticks,
/// Which, if is in microseconds, is about 10 septillion 780 sextillion years
pub fn get_tick_count() -> u128 {
    *TICK_COUNT.lock()
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
