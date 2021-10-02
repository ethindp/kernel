use bit_field::BitField;
use log::*;
use x86_64::instructions::{interrupts::without_interrupts, port::Port};

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

fn read(index: u8) -> u8 {
    let mut idx = Port::<u8>::new(IDX);
    let mut data = Port::<u8>::new(DATA);
    unsafe {
        idx.write(index | NO_NMI);
        data.read()
    }
}

fn write(index: u8, val: u8) {
    let mut idx = Port::<u8>::new(IDX);
    let mut data = Port::<u8>::new(DATA);
    unsafe {
        idx.write(index | NO_NMI);
        data.write(val);
    }
}

fn mask(index: u8, off: u8, on: u8) {
    let index = index | NO_NMI;
    let mut idx = Port::<u8>::new(IDX);
    let mut data = Port::<u8>::new(DATA);
    unsafe {
        idx.write(index);
        let val = data.read();
        data.write((val & !off) | on);
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
        if !read(STTSA).get_bit(7) {
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
            if !read(STTSA).get_bit(7) {
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
    let sttsb = read(STTSB);
    if !sttsb.get_bit(2) {
        second = (second & 0x0F) + ((second / 16) * 10);
        minute = (minute & 0x0F) + ((minute / 16) * 10);
        hour = ((hour & 0x0F) + (((hour & 0x70) / 16) * 10)) | (hour & 0x80);
        day = (day & 0x0F) + ((day / 16) * 10);
        month = (month & 0x0F) + ((month / 16) * 10);
        year = (year & 0x0F) + ((year / 16) * 10);
        century = (century & 0x0F) + ((century / 16) * 10);
        if sttsb.get_bit(1) && hour.get_bit(7) {
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
