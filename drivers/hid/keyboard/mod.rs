// SPDX-License-Identifier: MPL-2.0
use crate::printkln;
use cpuio::*;
use lazy_static::lazy_static;
use pc_keyboard::KeyCode;
use spin::RwLock;

lazy_static! {
    static ref KEY_QUEUE: RwLock<[Option<KeyCode>; 512]> = RwLock::new([None; 512]);
    static ref CHR_QUEUE: RwLock<[Option<char>; 512]> = RwLock::new([None; 512]);
    static ref CMD_QUEUE: RwLock<[Option<u8>; 512]> = RwLock::new([None; 512]);
    static ref RESEND_QUEUE: RwLock<[Option<u8>; 512]> = RwLock::new([None; 512]);
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
    let mut cmdqueue = CMD_QUEUE.write();
    let mut rsndqueue = RESEND_QUEUE.write();
    let mut queued_in_cmdqueue = false;
    let mut queued_in_rsndqueue = false;
    for it in cmdqueue.iter_mut().zip(rsndqueue.iter_mut()) {
        let (cmdq, rsndq) = it;
        if cmdq.is_none() && !queued_in_cmdqueue {
            *cmdq = Some(command);
            queued_in_cmdqueue = true;
        }
        if rsndq.is_none() && !queued_in_rsndqueue {
            *rsndq = Some(command);
            queued_in_rsndqueue = true;
        }
        if queued_in_cmdqueue && queued_in_rsndqueue {
            break;
        }
    }
}

pub fn dequeue_command() -> Option<u8> {
    let mut queue = CMD_QUEUE.write();
    for cmd in queue.iter_mut() {
        if cmd.is_some() {
            return cmd.take();
        }
    }
    None
}

pub fn notify_ack(byte: u8) {
    let mut cmdqueue = CMD_QUEUE.write();
    let mut resendqueue = RESEND_QUEUE.write();
    for cmd in cmdqueue.iter_mut() {
        if cmd.contains(&byte) {
            cmd.take();
            break;
        }
    }
    // Is this command in the resend queue? If so, find it and eliminate it.
    for cmd in resendqueue.iter_mut() {
        if cmd.contains(&byte) {
            cmd.take();
            break;
        }
    }
}

pub fn notify_resend(byte: u8) {
    let mut cmdqueue = CMD_QUEUE.write();
    let mut resendqueue = RESEND_QUEUE.write();
    for it in cmdqueue.iter_mut().zip(resendqueue.iter_mut()) {
        let (cmdbyte, rsndbyte) = it;
        if rsndbyte.contains(&byte) {
            rsndbyte.take();
        }
        if cmdbyte.is_none() {
            *cmdbyte = Some(byte);
            return;
        }
    }
    // This should never be reached. Ever.
    panic!("Can't locate byte {} in keyboard resend queue", byte);
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
    let (character, code) = key;
    if let Some(chr) = character {
        let mut chrqueue = CHR_QUEUE.write();
        for it in chrqueue.iter_mut() {
            if it.is_none() {
                *it = Some(chr);
                break;
            }
        }
    }
    if let Some(c) = code {
        let mut keyqueue = KEY_QUEUE.write();
        for it in keyqueue.iter_mut() {
            if it.is_none() {
                *it = Some(c);
                break;
            }
        }
    }
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
