// SPDX-License-Identifier: MPL-2.0
use cpuio::*;

pub fn pc_speaker_on(freq: u16) {
    let mut pitch: u32 = freq as u32;
    if pitch < 20 {
        pitch = 20;
    } else if pitch > 20000 {
        pitch = 20000;
    }
    unsafe {
        outb(0xb6, 0x43);
        outb((0x0012_34DD / pitch) as u8, 0x42);
        outb(((0x0012_34DD / pitch) >> 8) as u8, 0x42);
        outb(inb(0x61) | 0x03, 0x61);
    }
}

pub fn pc_speaker_off() {
    unsafe {
        outb(inb(0x61) & 0x03, 0x61);
    }
}
