use cpuio::*;

pub fn pc_speaker_on(freq: u16) {
    let mut pitch: u32 = freq as u32;
    let mut counter: u32 = 0;
    if pitch < 20 {
        pitch = 20;
    } else if pitch > 20000 {
        pitch = 20000;
    }
    counter = (0x1234DD / pitch).into();
    unsafe {
        outb(0x80 | 0x30 | 0x06 | 0x00, 0x43);
        outb((counter as u8) & 0xFF, 0x42);
        outb(((counter >> 8) as u8) & 0xFF, 0x42);
        outb(inb(0x61) | 0x01 | 0x02, 0x61);
    }
}

pub fn pc_speaker_off() {
    let status: u8 = unsafe { inb(0x61) };
    unsafe {
        outb(status & !(0x01 | 0x02), 0x61);
    }
}
