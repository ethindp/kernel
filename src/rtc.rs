use bitflags::bitflags;
use cpuio::{inb, outb};
use log::*;
use x86_64::instructions::interrupts::without_interrupts;

pub const IDX: u16 = 0x0070;
pub const DATA: u16 = 0x0071;
pub const NO_NMI: u8 = 0x80;
pub const SECS: u8 = 0x00;
pub const SECSALRM: u8 = 0x01;
pub const MINS: u8 = 0x02;
pub const MINSALRM: u8 = 0x03;
pub const HRS: u8 = 0x04;
pub const HRSALRM: u8 = 0x05;
pub const DAYWK: u8 = 0x06;
pub const DAYMO: u8 = 0x07;
pub const MON: u8 = 0x08;
pub const YR: u8 = 0x09;
pub const STTSA: u8 = 0x0A;
pub const STTSB: u8 = 0x0B;
pub const STTSC: u8 = 0x0C;
pub const STTSD: u8 = 0x0D;
pub const RST: u8 = 0x0F;
pub const FLP_DRV_TYP: u8 = 0x10;
pub const DSK_DAT: u8 = 0x12;
pub const EQ_INF: u8 = 0x14;
pub const DSK_DRV1_TYP: u8 = 0x19;
pub const DSK_DRV2_TYP: u8 = 0x1A;
pub const DSK_DRV1_CYL: u8 = 0x1B;
pub const DSK_DRV2_CYL: u8 = 0x24;
pub const MEM_EXT_LO: u8 = 0x30;
pub const MEM_EXT_HI: u8 = 0x31;
pub const CENT: u8 = 0x32;
pub const MEM_EXT2_LO: u8 = 0x34;
pub const MEM_EXT2_HI: u8 = 0x35;
pub const BIOS_BOOTFLG1: u8 = 0x38;
pub const BIOS_DSK_TRANS: u8 = 0x39;
pub const BIOS_BOOTFLG2: u8 = 0x3D;
pub const HIMEM_LO: u8 = 0x5B;
pub const HIMEM_MID: u8 = 0x5C;
pub const HIMEM_HI: u8 = 0x5D;
pub const BIOS_SMP_CNT: u8 = 0x5F;

bitflags! {
pub struct StatusA: u8 {
const UIP = 0x80;
}
}

bitflags! {
pub struct StatusB: u8 {
/// enable clock setting by freezing updates
const CLKSET = 1 << 7;
/// enable periodic interrupt
const PIE = 1 << 6;
/// enable alarm interrupt
const AIE = 1 << 5;
/// enable update-ended interrupt
const UEIE = 1 << 4;
/// enable square wave output
const SQWOE = 1 << 3;
/// Data Mode - 0: BCD, 1: Binary
const DATMD = 1 << 2;
/// 24/12 hour selection - 1 enables 24 hour mode
const HR24 = 1 << 1;
/// Daylight Savings Enable
const DSTE = 1 << 0;
}
}

bitflags! {
pub struct StatusC: u8 {
/// Interrupt request flag =1 when any or all of bits 6-4 are 1 and appropriate enables
/// (Register B) are set to 1. Generates IRQ 8 when triggered.
const IRQ = 1 << 7;
/// Periodic Interrupt flag
const PIRQ = 1 << 6;
/// Alarm Interrupt flag
const AI = 1 << 5;
/// Update-Ended Interrupt Flag
const UEI = 1 << 4;
}
}

bitflags! {
pub struct StatusD: u8 {
/// Valid RAM - 1 indicates battery power good, 0 if dead or disconnected.
const VRAM = 1 << 7;
}
}

pub fn read(index: u8) -> u8 {
    let idx = index | NO_NMI;
    unsafe {
        outb(idx, IDX);
        inb(DATA)
    }
}

pub fn write(index: u8, val: u8) {
    let idx = index | NO_NMI;
    unsafe {
        outb(idx, IDX);
        outb(val, DATA);
    }
}

pub fn mask(index: u8, off: u8, on: u8) {
    let index = index | NO_NMI;
    unsafe {
        outb(index, IDX);
        let val = inb(DATA);
        outb((val & !off) | on, DATA);
    }
}

#[allow(unused_results)]
pub async fn init() {
    info!("configuring RTC");
    without_interrupts(|| {
        write(STTSA, 0x26);
        mask(STTSB, !(1 << 0), 1 << 1);
        read(STTSC);
        read(STTSD);
    });
    info!(
        "RTC indicates that time is {}-{}-{} {}:{}:{}",
        read(YR),
        read(MON),
        read(DAYMO),
        read(HRS),
        read(MINS),
        read(SECS)
    );
}
