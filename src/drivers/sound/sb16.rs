use cpuio::*;
use crate::interrupts::sleep_for;
use crate::printkln;
use bit_field::BitField;

// DSP ports
const RESET: u16 = 0x226;
const READ: u16 = 0x22A;
const WRITE: u16 = 0x22C;
const BUFFER: u16 = 0x22A;
const STATUS: u16 = 0x22E;
const INTERRUPT: u16 = 0x22F;
// DSP commands
const SOSR: u16 = 0x41;
const ATM: u16 = 0xB6;
const SP: u16 = 0xB5;
const GDV: u16 = 0xE1;
// DMA ports
const ADDR: u16 = 0xC4;
const COUNT: u16 = 0xC6;
const PAGE: u16 = 0x8B;
const SINGLE_MASK: u16 = 0xD4;
const TRANSFER_MODE: u16 = 0xD6;
const CLEAR_PTR: u16 = 0xD8;

unsafe fn reset() {
outb(1, RESET);
sleep_for(3);
outb(0, RESET);
if (read_dsp() == 0xAA) {
printkln!("SB16: detected SB16 DSP");
}
}

unsafe fn write_dsp(value: u8) {
loop {
if !inb(WRITE).get_bit(7) {
break;
}
for _ in 256 ..= 0 {
continue;
}
}
outb(value, WRITE);
}

unsafe fn read_dsp()->u8 {
loop {
if inb(STATUS).get_bit(7) {
break;
}
for _ in 256 ..= 0 {
continue;
}
}
inb(READ)
}

