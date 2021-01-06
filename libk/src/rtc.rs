use bit_field::BitField;
use bitflags::bitflags;
use cpuio::{inb, outb};
use log::*;
use x86_64::instructions::interrupts::without_interrupts;

const IDX: u16 = 0x0070;
const DATA: u16 = 0x0071;
const NO_NMI: u8 = 0x80;
const SECS: u8 = 0x00;
const SECSALRM: u8 = 0x01;
const MINS: u8 = 0x02;
const MINSALRM: u8 = 0x03;
const HRS: u8 = 0x04;
const HRSALRM: u8 = 0x05;
const DAYWK: u8 = 0x06;
const DAYMO: u8 = 0x07;
const MON: u8 = 0x08;
const YR: u8 = 0x09;
const STTSA: u8 = 0x0A;
const STTSB: u8 = 0x0B;
const STTSC: u8 = 0x0C;
const STTSD: u8 = 0x0D;
const RST: u8 = 0x0F;
const FLP_DRV_TYP: u8 = 0x10;
const DSK_DAT: u8 = 0x12;
const EQ_INF: u8 = 0x14;
const DSK_DRV1_TYP: u8 = 0x19;
const DSK_DRV2_TYP: u8 = 0x1A;
const DSK_DRV1_CYL: u8 = 0x1B;
const DSK_DRV2_CYL: u8 = 0x24;
const MEM_EXT_LO: u8 = 0x30;
const MEM_EXT_HI: u8 = 0x31;
const CENT: u8 = 0x32;
const MEM_EXT2_LO: u8 = 0x34;
const MEM_EXT2_HI: u8 = 0x35;
const BIOS_BOOTFLG1: u8 = 0x38;
const BIOS_DSK_TRANS: u8 = 0x39;
const BIOS_BOOTFLG2: u8 = 0x3D;
const HIMEM_LO: u8 = 0x5B;
const HIMEM_MID: u8 = 0x5C;
const HIMEM_HI: u8 = 0x5D;
const BIOS_SMP_CNT: u8 = 0x5F;
const CURYR: u128 = 2020;

bitflags! {
struct StatusA: u8 {
/// Update in progress
const UIP = 0x80;
}
}

bitflags! {
struct StatusB: u8 {
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
struct StatusC: u8 {
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
struct StatusD: u8 {
/// Valid RAM - 1 indicates battery power good, 0 if dead or disconnected.
const VRAM = 1 << 7;
}
}

fn read(index: u8) -> u8 {
    let idx = index | NO_NMI;
    unsafe {
        outb(idx, IDX);
        inb(DATA)
    }
}

fn write(index: u8, val: u8) {
    let idx = index | NO_NMI;
    unsafe {
        outb(idx, IDX);
        outb(val, DATA);
    }
}

fn mask(index: u8, off: u8, on: u8) {
    let index = index | NO_NMI;
    unsafe {
        outb(index, IDX);
        let val = inb(DATA);
        outb((val & !off) | on, DATA);
    }
}

/// Initializes the RTC subsystem.
pub async fn init() {
    info!("configuring RTC");
    without_interrupts(|| {
        write(STTSA, 0x26);
        mask(STTSB, !(1 << 0), 1 << 1);
        let _ = read(STTSC);
        let _ = read(STTSD);
        let prev = read(STTSB);
        write(STTSB, prev | 0x40);
    });
    let (year, month, day, hour, minute, second, _) = current_time();
    info!(
        "Current time: {}-{}-{} {}:{}:{}",
        year, month, day, hour, minute, second
    );
}

/// Returns the current time in a tuple of (year, month, day, hour, minute, second, century).
pub fn current_time() -> (u128, u128, u128, u128, u128, u128, u128) {
    loop {
        if !StatusA::from_bits_truncate(read(STTSA)).contains(StatusA::UIP) {
            break;
        }
    }
    let mut second = read(SECS) as u128;
    let mut minute = read(MINS) as u128;
    let mut hour = read(HRS) as u128;
    let mut day = read(DAYMO) as u128;
    let mut month = read(MON) as u128;
    let mut year = read(YR) as u128;
    let mut century = read(CENT) as u128;
    let (mut lsec, mut lmin, mut lhr, mut lday, mut lmo, mut lyr, mut lcent);
    loop {
        lsec = second;
        lmin = minute;
        lhr = hour;
        lday = day;
        lmo = month;
        lyr = year;
        lcent = century;
        loop {
            if !StatusA::from_bits_truncate(read(STTSA)).contains(StatusA::UIP) {
                break;
            }
        }
        second = read(SECS) as u128;
        minute = read(MINS) as u128;
        hour = read(HRS) as u128;
        day = read(DAYMO) as u128;
        month = read(MON) as u128;
        year = read(YR) as u128;
        century = read(CENT) as u128;
        if (lsec != second)
            || (lmin != minute)
            || (lhr != hour)
            || (lday != day)
            || (lmo != month)
            || (lyr != year)
            || (lcent != century)
        {
            break;
        }
    }
    let sttsb = StatusB::from_bits(read(STTSB)).unwrap();
    if sttsb.contains(StatusB::DATMD) {
        second = (second & 0x0F) + ((second / 16) * 10);
        minute = (minute & 0x0F) + ((minute / 16) * 10);
        hour = ((hour & 0x0F) + (((hour & 0x70) / 16) * 10)) | (hour & 0x80);
        day = (day & 0x0F) + ((day / 16) * 10);
        month = (month & 0x0F) + ((month / 16) * 10);
        year = (year & 0x0F) + ((year / 16) * 10);
        century = (century & 0x0F) + ((century / 16) * 10);
        if sttsb.contains(StatusB::HR24) && hour.get_bit(7) {
            hour = ((hour & 0x7F) + 12) % 24;
        }
    }
    if century * 100 == CURYR {
        year += century * 100;
    } else {
        year += (CURYR / 100) * 100;
        if year < CURYR {
            year += 100;
        }
    }
    (year, month, day, hour, minute, second, century)
}
