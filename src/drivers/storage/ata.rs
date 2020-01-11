//pub mod dco;
pub mod smart;
//pub mod security;
//pub mod identification;
use cpuio::*;
//use crate::interrupts::sleep_for;
use crate::printkln;
use alloc::collections::linked_list::LinkedList;
use bit_field::BitField;
use x86_64::instructions::hlt;

#[repr(u8)]
pub enum ATACommand {
    CfaEraseSectors = 0xC0,
    CfaRequestExtendedErrorCode = 0x03,
    CfaTranslateSector = 0x87,
    CfaWriteMultipleWithoutErase = 0xCD,
    CfaWriteSectorsWithoutErase = 0x38,
    CheckMediaCardType = 0xD1,
    CheckPowerMode = 0xE5,
    ConfigureStream = 0x51,
    DeviceConfigure = 0xB1,
    DeviceReset = 0x08,
    DownloadMicrocode = 0x92,
    ExecuteDeviceDiagnostic = 0x90,
    FlushCache = 0xE7,
    FlushCacheExt = 0xEA,
    IdentifyDevice = 0xEC,
    IdentifyPacketDevice = 0xA1,
    Idle = 0xE3,
    IdleImmediate = 0xE1,
    Nop = 0x00,
    NvCache = 0xB6,
    Packet = 0xA0,
    ReadBuffer = 0xE4,
    ReadDma = 0xC8,
    ReadDmaExt = 0x25,
    ReadDmaQueued = 0xC7,
    ReadDmaQueuedExt = 0x26,
    ReadFpdmaQueued = 0x60,
    ReadLogExt = 0x2F,
    ReadLogDmaExt = 0x47,
    ReadMultiple = 0xC4,
    ReadMultipleExt = 0x29,
    ReadNativeMaxAddress = 0xF8,
    ReadNativeMaxAddressExt = 0x27,
    ReadSectors = 0x20,
    ReadSectorsExt = 0x24,
    ReadStreamDmaExt = 0x2A,
    ReadStreamExt = 0x2B,
    ReadVerifySectors = 0x40,
    ReadVerifySectorsExt = 0x42,
    SecurityDisablePassword = 0xF6,
    SecurityErasePrepare = 0xF3,
    SecurityEraseUnit = 0xF4,
    SecurityFrezeLock = 0xF5,
    SecuritySetPassword = 0xF1,
    SecurityUnlock = 0xF2,
    Service = 0xA2,
    SetFeatures = 0xEF,
    SetMax = 0xF9,
    SetMaxAddressExt = 0x37,
    SetMultipleMode = 0xC6,
    Sleep = 0xE6,
    Smart = 0xB0,
    Standby = 0xE2,
    StandbyImmediate = 0xE0,
    TrustedNonData = 0x5B,
    TrustedReceive = 0x5C,
    TrustedReceiveDma = 0x5D,
    TrustedSend = 0x5E,
    TrustedSendDma = 0x5F,
    WriteBuffer = 0xE8,
    WriteDma = 0xCA,
    WriteDmaExt = 0x35,
    WriteDmaFuaExt = 0x3D,
    WriteDmaQueued = 0xCC,
    WriteDmaQueuedExt = 0x36,
    WriteDmaQueuedFuaExt = 0x3E,
    WriteFpdmaQueued = 0x61,
    WriteLogExt = 0x3F,
    WriteLogDmaExt = 0x57,
    WriteMultiple = 0xC5,
    WriteMultipleExt = 0x39,
    WriteMultipleFuaExt = 0xCE,
    WriteSectors = 0x30,
    WriteSectorsExt = 0x34,
    WriteStreamDmaExt = 0x3A,
    WriteStreamExt = 0x3B,
    WriteUncorrectableExt = 0x45,
}

#[repr(u8)]
pub enum DCOSubcommand {
    FreezeLock = 0xC1,
    Identify = 0xC2,
    Restore = 0xC0,
    Set = 0xC3,
}

#[repr(u8)]
pub enum NvCacheSubcommand {
    AddLbasToPinnedSet = 0x10,
    Flush = 0x14,
    Disable = 0x16,
    Enable = 0x15,
    QueryMisses = 0x13,
    QueryPinnedSet = 0x12,
    RemoveLbasFromPinnedSet = 0x11,
    ReturnFromPowerMode = 0x01,
    SetPowerMode = 0x00,
}

#[repr(u8)]
pub enum SmartSubcommand {
    DisableOperations = 0xD9,
    ToggleAttributeAutosave = 0xD2,
    EnableOperations = 0xD8,
    ExecuteOfflineImmediate = 0xD4,
    ReadData = 0xD0,
    ReadLog = 0xD5,
    ReturnStatus = 0xDA,
    WriteLog = 0xD6,
}

// ATA ports (primary)
const DATA: u16 = 0x1F0;
const ERROR: u16 = 0x1F1;
const FEATURES: u16 = 0x1F1;
const SECTOR_COUNT: u16 = 0x1F2;
const LBAL: u16 = 0x1F3;
const LBAM: u16 = 0x1F4;
const LBAH: u16 = 0x1F5;
const DRIVESEL: u16 = 0x1F6;
const STATUS: u16 = 0x1F7;
const COMMAND: u16 = 0x1F7;
const ALTSTATUS: u16 = 0x3F6;
const DEVCTL: u16 = 0x3F6;
const DRIVE_ADDR: u16 = 0x3F7;
// ATA ports (secondary)
const DATA2: u16 = 0x170;
const ERROR2: u16 = 0x171;
const FEATURES2: u16 = 0x171;
const SECTOR_COUNT2: u16 = 0x172;
const LBAL2: u16 = 0x173;
const LBAM2: u16 = 0x174;
const LBAH2: u16 = 0x175;
const DRIVESEL2: u16 = 0x176;
const STATUS2: u16 = 0x177;
const COMMAND2: u16 = 0x177;
const ALTSTATUS2: u16 = 0x376;
const DEVCTL2: u16 = 0x376;
const DRIVE_ADDR2: u16 = 0x377;
// Error register bits
const AMNF: usize = 0;
const TKZNF: usize = 1;
const ABRT: usize = 2;
const MCR: usize = 3;
const IDNF: usize = 4;
const MC: usize = 5;
const UNC: usize = 6;
const BBK: usize = 7;
// Drive/head register bits (no ranges)
const DRV: usize = 4;
const LBA: usize = 6;
// Status register bits
const ERR: usize = 0;
const DRQ: usize = 3;
const SRV: usize = 4;
const DF: usize = 5;
const RDY: usize = 6;
const BSY: usize = 7;
// DEVCTL bits
const NEIN: usize = 1;
const SRST: usize = 2;
const HOB: usize = 7;
// Drive address register bits
const DS0: usize = 0;
const DS1: usize = 1;
const HS0: usize = 2;
const HS1: usize = 3;
const HS2: usize = 4;
const HS3: usize = 5;
const WTG: usize = 6;

pub fn init() {
    // We use no PCI enumeration code here
    let mut drive_cnt = 0;
    unsafe {
        if inb(STATUS) == 0xFF {
            printkln!("ATA: bus 0 has no master");
        } else {
            drive_cnt += 1;
            let mut cmd = 0u8;
            cmd.set_bit(2, true);
            outb(cmd, DEVCTL);
            cmd.set_bit(2, false);
            outb(cmd, DEVCTL);
            cmd.set_bit(1, true);
            outb(cmd, DEVCTL);
        }
        if inb(STATUS2) == 0xFF {
            printkln!("ATA: bus 0 has no slave");
        } else {
            drive_cnt += 1;
            let mut cmd = 0u8;
            cmd.set_bit(2, true);
            outb(cmd, DEVCTL2);
            cmd.set_bit(2, false);
            outb(cmd, DEVCTL2);
            cmd.set_bit(1, true);
            outb(cmd, DEVCTL2);
        }
    }
    if drive_cnt == 0 {
        printkln!("ATA: no ATA drives available; aborting initialization sequence");
        return;
    }
    printkln!("ATA: identifying drive 0");
    if let Some(data) = unsafe { identify_device_raw(1) } {
        if data[83].get_bit(10) {
            printkln!("ATA: drive 0: LBA 48 supported");
            let tsecs = ((data[103] as u64) << 48)
                + ((data[102] as u64) << 32)
                + ((data[101] as u64) << 16)
                + data[100] as u64;
            printkln!("ATA: drive 0: max sectors = {}", tsecs);
        } else {
            printkln!("ATA: drive 0: LBA 28 is supported");
            let tsecs = ((data[60] as u64) << 16) + data[61] as u64;
            printkln!("ATA: drive 0: max sectors = {}", tsecs);
        }
    } else {
        panic!("ATA: drive 0 set ERR bit");
    }
}

pub unsafe fn read_sectors_ext(drive: u8, lba: u64, count: u16) -> [u8; 512] {
    match drive {
        0 => outb(0xE0, DRIVESEL),
        1 => outb(0xF0, DRIVESEL),
        2 => outb(0xE0, DRIVESEL2),
        3 => outb(0xF0, DRIVESEL2),
        d => panic!(
            "ATA: read sector(s) extended got invalid drive number {}",
            d
        ),
    }
    if drive == 0 || drive == 1 {
        outb(count.get_bits(8..=15) as u8, SECTOR_COUNT);
        outb(lba.get_bits(24..32) as u8, LBAL);
        outb(lba.get_bits(32..40) as u8, LBAM);
        outb(lba.get_bits(40..48) as u8, LBAH);
        outb(count.get_bits(0..8) as u8, SECTOR_COUNT);
        outb(lba.get_bits(0..8) as u8, LBAL);
        outb(lba.get_bits(8..16) as u8, LBAM);
        outb(lba.get_bits(16..24) as u8, LBAH);
        outb(ATACommand::ReadSectorsExt as u8, COMMAND);
        while inb(STATUS).get_bit(BSY) {
            hlt();
        }
        let mut bytes: LinkedList<u8> = LinkedList::new();
        for _ in 0..256 {
            let rawbytes = inw(DATA).to_le_bytes();
            bytes.push_back(rawbytes[0]);
            bytes.push_back(rawbytes[1]);
        }
        let mut sector: [u8; 512] = [0; 512];
        for it in bytes.iter().zip(sector.iter_mut()) {
            let (byte, sector) = it;
            *sector = *byte;
        }
        drop(bytes);
        return sector;
    } else if drive == 3 || drive == 4 {
        outb(count.get_bits(8..=15) as u8, SECTOR_COUNT2);
        outb(lba.get_bits(24..32) as u8, LBAL2);
        outb(lba.get_bits(32..40) as u8, LBAM2);
        outb(lba.get_bits(40..48) as u8, LBAH2);
        outb(count.get_bits(0..8) as u8, SECTOR_COUNT2);
        outb(lba.get_bits(0..8) as u8, LBAL2);
        outb(lba.get_bits(8..16) as u8, LBAM2);
        outb(lba.get_bits(16..24) as u8, LBAH2);
        outb(ATACommand::ReadSectorsExt as u8, COMMAND2);
        while inb(STATUS2).get_bit(BSY) {
            hlt();
        }
        let mut bytes: LinkedList<u8> = LinkedList::new();
        for _ in 0..256 {
            let rawbytes = inw(DATA2).to_le_bytes();
            bytes.push_back(rawbytes[0]);
            bytes.push_back(rawbytes[1]);
        }
        let mut sector: [u8; 512] = [0; 512];
        for it in bytes.iter().zip(sector.iter_mut()) {
            let (byte, sector) = it;
            *sector = *byte;
        }
        drop(bytes);
        return sector;
    } else {
        return [0u8; 512];
    }
}

pub unsafe fn identify_device_raw(drive: u8) -> Option<[u16; 256]> {
    match drive {
        0 => outb(0xE0, DRIVESEL),
        1 => outb(0xF0, DRIVESEL),
        2 => outb(0xE0, DRIVESEL2),
        3 => outb(0xF0, DRIVESEL2),
        d => panic!("ATA: identify device got invalid drive number {}", d),
    }
    if drive == 0 || drive == 1 {
        outb(0, SECTOR_COUNT);
        outb(0, LBAL);
        outb(0, LBAM);
        outb(0, LBAH);
        outb(ATACommand::IdentifyDevice as u8, COMMAND);
        if inb(STATUS) == 0 {
            return None;
        }
        while inb(STATUS).get_bit(BSY) {
            hlt();
        }
        if (inb(LBAM) > 0 || inb(LBAH) > 0) || (inb(LBAM) > 0 && inb(LBAH) > 0) {
            return None;
        }
        if !inb(STATUS).get_bit(ERR) && inb(STATUS).get_bit(DRQ) {
            let mut data: [u16; 256] = [0; 256];
            for item in data.iter_mut() {
                *item = inw(DATA);
            }
            Some(data)
        } else {
            return None;
        }
    } else if drive == 3 || drive == 4 {
        outb(0, SECTOR_COUNT2);
        outb(0, LBAL2);
        outb(0, LBAM2);
        outb(0, LBAH2);
        outb(ATACommand::IdentifyDevice as u8, COMMAND2);
        if inb(STATUS2) == 0 {
            return None;
        }
        while inb(STATUS2).get_bit(BSY) {
            hlt();
        }
        if (inb(LBAM2) > 0 || inb(LBAH2) > 0) || (inb(LBAM2) > 0 && inb(LBAH2) > 0) {
            return None;
        }
        if !inb(STATUS2).get_bit(ERR) && inb(STATUS2).get_bit(DRQ) {
            let mut data: [u16; 256] = [0; 256];
            for item in data.iter_mut() {
                *item = inw(DATA2);
            }
            Some(data)
        } else {
            return None;
        }
    } else {
        return None;
    }
}
