extern crate alloc;
use crate::printkln;
use alloc::collections::VecDeque;
use cpuio::*;
use lazy_static::lazy_static;
use pc_keyboard::KeyCode;
use spin::Mutex;

lazy_static! {
    static ref KEY_QUEUE: Mutex<VecDeque<(Option<char>, Option<KeyCode>)>> =
        Mutex::new(VecDeque::new());
    static ref CMD_QUEUE: Mutex<VecDeque<u8>> = Mutex::new(VecDeque::new());
    static ref RESEND_QUEUE: Mutex<VecDeque<u8>> = Mutex::new(VecDeque::new());
}

pub fn init() {
    printkln!("KBD: initializing");
    printkln!("KBD: initiating self test");
    disable();
    queue_command(0xFF);
    enable();
    printkln!("KBD: identifying keyboard");
    queue_command(0xF2);
    printkln!("KBD: initialization complete");
}

fn queue_command(command: u8) {
    CMD_QUEUE.lock().push_back(command);
    RESEND_QUEUE.lock().push_back(command);
}

pub fn dequeue_command() -> Option<u8> {
    CMD_QUEUE.lock().pop_front()
}

pub fn notify_ack(byte: u8) {
    let mut idx = usize::max_value();
    for (i, cmd) in CMD_QUEUE.lock().iter().enumerate() {
        if *cmd == byte {
            idx = i;
            break;
        }
    }
    if idx < usize::max_value() {
        CMD_QUEUE.lock().remove(idx);
    }
    // Is this command in the resend queue? If so, find it and eliminate it.
    idx = usize::max_value();
    for (i, cmd) in RESEND_QUEUE.lock().iter().enumerate() {
        if *cmd == byte {
            idx = i;
            break;
        }
    }
    if idx < usize::max_value() {
        RESEND_QUEUE.lock().remove(idx);
    }
}

pub fn notify_resend(byte: u8) {
    // If we got here, the command should *not* be in the command queue.
    // Cover this case anyway.
    let mut idx = usize::max_value();
    for (i, cmd) in CMD_QUEUE.lock().iter().enumerate() {
        if *cmd == byte {
            idx = i;
            break;
        }
    }
    if idx <= usize::max_value() {
        CMD_QUEUE.lock().remove(idx);
    }
    // The command should be in the resend queue, though.
    for cmd in RESEND_QUEUE.lock().iter() {
        if *cmd == byte {
            CMD_QUEUE.lock().push_back(*cmd);
            break;
        } else {
            // This shouldn't ever happen, but we need to handle it anyway.
            panic!(
                "KBD: kernel notified driver that byte {:X} required resend, but couldn't find it",
                byte
            );
        }
    }
}

pub fn notify_key_error() {
    printkln!("KBD: warning: detected key detection error or internal buffer overrun");
}

pub fn notify_self_test_succeeded() {
    printkln!("KBD: self test OK");
}

pub fn notify_self_test_failed() {
    panic!("KBD: error: self test failed!");
}

pub fn notify_key(key: (Option<char>, Option<KeyCode>)) {
    KEY_QUEUE.lock().push_back(key);
}

pub fn notify_id_finished(byte1: u8, byte2: u8) {
    match (byte1, byte2) {
        (0xFA, 0xFA) | (0xFA, 0x00) => {
            printkln!("KBD: detected ancient AT keyboard");
        }
        (0x00, 0x00) => {
            printkln!("KBD: detected standard PS/2 mouse");
        }
        (0x03, 0x00) => {
            printkln!("KBD: detected mouse with scroll wheel");
        }
        (0x04, 0x00) => {
            printkln!("KBD: detected 5-button mouse");
        }
        (0xAB, 0x41) | (0xAB, 0xC1) => {
            printkln!("KBD: detected multifunction PS/2 keyboard with translation");
        }
        (0xAB, 0x83) => {
            printkln!("KBD: detected standard PS/2 keyboard");
        }
        (byte1, byte2) => {
            printkln!(
                "KBD: detected unknown keyboard with ID {:X}, {:X}",
                byte1,
                byte2
            );
        }
    }
}

// The below functions are direct interfaces to the keyboard, as we cannot rely on the above interfaces for keyboard control.
// These functions are unsafe because they directly send and receive bytes to and from the keyboard, which can cause undefined behavior.

/// Set status LEDs - This command can be used to turn on and off the Num Lock, Caps Lock and Scroll Lock LEDs. After receiving this command, the keyboard will reply with an ACK (FA) and wait for another byte which determines the status of the LEDs. Bit 0 controls the Scroll Lock, bit 1 controls the Num Lock and Bit 2 controls the Caps lock. Bits 3 to 7 are ignored.
unsafe fn set_leds_direct(leds: u8) {
    outb(0xED, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xED, 0x60);
        rsp = inb(0x60);
    }
    outb(leds, 0x60);
}

/// Set scan code set - Upon receiving F0, the keyboard will reply with an ACK (FA) and wait for another byte. This byte can be in the range 01 to 03, and it determines the scan code set to be used. Sending 00 as the second byte will return the scan code set currently in use.
unsafe fn set_scan_code_set_direct(scan_code_set: u8) -> Option<u8> {
    outb(0xF0, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xF0, 0x60);
        rsp = inb(0x60);
    }
    if scan_code_set == 0x00 {
        outb(scan_code_set, 0x60);
        Some(inb(0x60))
    } else {
        outb(scan_code_set, 0x60);
        None
    }
}

/// Set repeat rate - The keyboard will acknowledge the command with an ACK (FA) and wait for the second byte which determines the repeat rate.
unsafe fn set_repeat_rate_direct(rate: u8) {
    outb(0xF3, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xF3, 0x60);
        rsp = inb(0x60);
    }
    outb(rate, 0x60);
}

/// Set Key Type Make - Disable break codes and typematic repeat for specified keys.  Keyboard responds with "ack" (0xFA), then disables scanning (if enabled) and reads a list of keys from the host.  These keys are specified by their set 3 make codes.  Keyboard responds to each make code with "ack".  Host terminates this list by sending an invalid set 3 make code (eg, a valid command.)  The keyboard then re-enables scanning (if previously disabled).
unsafe fn set_key_type_make_direct(keys: &[u8]) {
    outb(0xFD, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xFD, 0x60);
        rsp = inb(0x60);
    }
    for key in keys {
        outb(*key, 0x60);
    }
    outb(0xFF, 0x60);
}

/// Set Key Type Make/Break - Similar to previous command, except this one only disables typematic repeat.
unsafe fn set_key_type_make_break_direct(keys: &[u8]) {
    outb(0xFC, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xFC, 0x60);
        rsp = inb(0x60);
    }
    for key in keys {
        outb(*key, 0x60);
    }
    outb(0xFF, 0x60);
}

/// Set Key Type Typematic - Similar to previous two, except this one only disables break codes.
unsafe fn set_key_type_typematic_direct(keys: &[u8]) {
    outb(0xFB, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xFB, 0x60);
        rsp = inb(0x60);
    }
    for key in keys {
        outb(*key, 0x60);
    }
    outb(0xFF, 0x60);
}

/// Set All Keys Typematic/Make/Break - Keyboard responds with "ack" (0xFA).  Sets all keys to their normal setting (generate scan codes on make, break, and typematic repeat).
unsafe fn set_all_keys_typematic_make_break_direct() {
    outb(0xFA, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xFA, 0x60);
        rsp = inb(0x60);
    }
}

/// Set All Keys Make - Keyboard responds with "ack" (0xFA).  Similar to 0xFD, except applies to all keys.
unsafe fn set_all_keys_make_direct() {
    outb(0xF9, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xF9, 0x60);
        rsp = inb(0x60);
    }
}

/// Set All Keys Make/Break - Keyboard responds with "ack" (0xFA).  Similar to 0xFC, except applies to all keys.
unsafe fn set_all_keys_make_break_direct() {
    outb(0xF8, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xF8, 0x60);
        rsp = inb(0x60);
    }
}

/// Set All Keys Typematic - Keyboard responds with "ack" (0xFA).  Similar to 0xFB, except applies to all keys.
unsafe fn set_all_keys_typematic_direct() {
    outb(0xF7, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xF7, 0x60);
        rsp = inb(0x60);
    }
}

/// Set Default - Load default typematic rate/delay (10.9cps / 500ms), key types (all keys typematic/make/break), and scan code set (2).
unsafe fn set_default_direct() {
    outb(0xF6, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xF6, 0x60);
        rsp = inb(0x60);
    }
}

/// Disable - Keyboard stops scanning, loads default values (see "Set Default" command), and waits further instructions.
unsafe fn disable_direct() {
    outb(0xF5, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xF5, 0x60);
        rsp = inb(0x60);
    }
}

/// Enable - Re-enables keyboard after disabled using previous command.
unsafe fn enable_direct() {
    outb(0xF4, 0x60);
    let mut rsp = inb(0x60);
    while rsp != 0xFA {
        outb(0xF4, 0x60);
        rsp = inb(0x60);
    }
}

// Safe interfaces.
pub fn set_leds(leds: u8) {
    unsafe {
        set_leds_direct(leds);
    }
}

pub fn set_scancode_set(set: u8) -> Option<u8> {
    unsafe { set_scan_code_set_direct(set) }
}

pub fn set_typematic_rate_delay(rate_delay: u8) {
    unsafe {
        set_repeat_rate_direct(rate_delay);
    }
}

pub fn set_key_type_make(keys: &[u8]) {
    unsafe {
        set_key_type_make_direct(keys);
    }
}

pub fn set_key_type_make_break(keys: &[u8]) {
    unsafe {
        set_key_type_make_break_direct(keys);
    }
}

pub fn set_key_type_typematic(keys: &[u8]) {
    unsafe {
        set_key_type_typematic_direct(keys);
    }
}

pub fn set_all_keys_typematic_make_break() {
    unsafe {
        set_all_keys_typematic_make_break_direct();
    }
}

pub fn set_all_keys_make() {
    unsafe {
        set_all_keys_make_direct();
    }
}

pub fn set_all_keys_make_break() {
    unsafe {
        set_all_keys_make_break_direct();
    }
}

pub fn set_all_keys_typematic() {
    unsafe {
        set_all_keys_typematic_direct();
    }
}

pub fn set_default() {
    unsafe {
        set_default_direct();
    }
}

pub fn disable() {
    unsafe {
        disable_direct();
    }
}

pub fn enable() {
    unsafe {
        enable_direct();
    }
}

/// Returns a key code if keys are in the internal key queue, otherwise returns None.
pub fn read() -> Option<(Option<char>, Option<KeyCode>)> {
    KEY_QUEUE.lock().pop_front()
}
