//pub mod dco;
pub mod smart;
//pub mod security;
pub mod identification;
use cpuio::*;
//use crate::interrupts::sleep_for;
use crate::printkln;
use alloc::collections::linked_list::LinkedList;
use alloc::string::String;
use alloc::vec::Vec;
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
    let identification = identify_device(1);
    if identification.cmd_ft_sets.lba48 {
        printkln!("ATA: drive supports 48-bit LBA addressing");
    } else {
        printkln!("ATA: drive supports 28-bit LBA adressing");
    }
    printkln!(
        "ATA: maximum 28-bit LBA address = {} ({:X}h)",
        identification.total_sectors_lba28,
        identification.total_sectors_lba28
    );
    printkln!(
        "ATA: maximum 48-bit LBA address = {} ({:X}h)",
        identification.total_sectors_lba48,
        identification.total_sectors_lba48
    );
    printkln!("ATA: Drive serial number: {}", identification.sn);
    printkln!("ATA: Firmware revision: {}", identification.fw_rev);
    printkln!("ATA: model number: {}", identification.model_number);
    if identification.cmd_ft_sets.media_sn && identification.en_cmd_ft_sets.valid_media_sn {
        printkln!(
            "ATA: media serial number: {}",
            identification.current_media_sn
        );
    } else {
        printkln!("ATA: no media serial number available");
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

pub fn identify_device(drive: u8) -> identification::DeviceIdentification {
    // unwrap() is used here because the identify_device_raw() function will never return None
    let raw = unsafe { identify_device_raw(drive).unwrap() };
    // Verify checksum
    if raw[255].get_bits(0..8) as u8 == 0xA5 as u8 {
        let mut checksum = 0u8;
        for i in 0..255 {
            checksum += raw[i] as u8;
        }
        checksum += raw[255].get_bits(0..8) as u8;
        if checksum != raw[255].get_bits(8..16) as u8 {
            panic!(
                "ATA: checksum verification failure: got {}, expected {}",
                checksum,
                raw[255].get_bits(8..16)
            );
        }
    } else {
        printkln!("ATA: warning: cannot verify checksum: device is not ATA8 ACS compliant");
        printkln!(
            "ATA: warning: bits 7:0 of word 255 == {:X}, should be 0xA5",
            raw[255].get_bits(0..8)
        );
    }
    // Assemble strings
    let sn = {
        let mut bytes: Vec<u8> = Vec::new();
        for i in 10..20 {
            let part = raw[i].to_le_bytes();
            bytes.push(part[0]);
            bytes.push(part[1]);
        }
        // Swap the bytes
        for i in (0..bytes.len()).step_by(2) {
            let tmp = bytes[i];
            bytes[i] = bytes[i + 1];
            bytes[i + 1] = tmp;
        }
        String::from_utf8(bytes).unwrap()
    };
    let fw_rev = {
        let mut bytes: Vec<u8> = Vec::new();
        for i in 23..27 {
            let part = raw[i].to_le_bytes();
            bytes.push(part[0]);
            bytes.push(part[1]);
        }
        // Swap the bytes
        for i in (0..bytes.len()).step_by(2) {
            let tmp = bytes[i];
            bytes[i] = bytes[i + 1];
            bytes[i + 1] = tmp;
        }
        String::from_utf8(bytes).unwrap()
    };
    let model_number = {
        let mut bytes: Vec<u8> = Vec::new();
        for i in 27..47 {
            let part = raw[i].to_le_bytes();
            bytes.push(part[0]);
            bytes.push(part[1]);
        }
        // Swap the bytes
        for i in (0..bytes.len()).step_by(2) {
            let tmp = bytes[i];
            bytes[i] = bytes[i + 1];
            bytes[i + 1] = tmp;
        }
        String::from_utf8(bytes).unwrap()
    };
    let current_media_sn = {
        let mut bytes: Vec<u8> = Vec::new();
        for i in 176..206 {
            let part = raw[i].to_le_bytes();
            bytes.push(part[0]);
            bytes.push(part[1]);
        }
        // Swap the bytes
        for i in (0..bytes.len()).step_by(2) {
            let tmp = bytes[i];
            bytes[i] = bytes[i + 1];
            bytes[i + 1] = tmp;
        }
        String::from_utf8(bytes).unwrap()
    };
    let gen_config = identification::GeneralConfiguration {
        is_ata: raw[0].get_bit(15),
        data_incomplete: raw[0].get_bit(2),
    };
    let capabilities = identification::Capabilities {
        standard_standby_timer: raw[49].get_bit(15),
        iordy_supported: raw[49].get_bit(11),
        iordy_adjustable: raw[49].get_bit(10),
        lba_supported: raw[49].get_bit(9),
        dma_supported: raw[49].get_bit(8),
        min_standby_vendor: raw[50].get_bit(0),
    };
    let selected_mwdma_mode = if raw[63].get_bit(10) {
        identification::MwDmaModeSelected { dma2: true }
    } else if raw[63].get_bit(9) {
        identification::MwDmaModeSelected { dma1: true }
    } else if raw[63].get_bit(8) {
        identification::MwDmaModeSelected { dma0: true }
    } else {
        identification::MwDmaModeSelected { unknown: true }
    };
    let speed = if raw[77].get_bits(1..=3) == 1 {
        identification::CurrentNegotiatedSpeed { gen1: true }
    } else if raw[77].get_bits(1..=3) == 2 {
        identification::CurrentNegotiatedSpeed { gen2: true }
    } else if raw[77].get_bits(1..=3) == 3 {
        identification::CurrentNegotiatedSpeed { gen3: true }
    } else {
        identification::CurrentNegotiatedSpeed { unknown: true }
    };
    let current_udma_mode = if raw[88].get_bit(14) {
        identification::SelectedUdmaMode { udma6: true }
    } else if raw[88].get_bit(13) {
        identification::SelectedUdmaMode { udma5: true }
    } else if raw[88].get_bit(12) {
        identification::SelectedUdmaMode { udma4: true }
    } else if raw[88].get_bit(11) {
        identification::SelectedUdmaMode { udma3: true }
    } else if raw[88].get_bit(10) {
        identification::SelectedUdmaMode { udma2: true }
    } else if raw[88].get_bit(9) {
        identification::SelectedUdmaMode { udma1: true }
    } else if raw[88].get_bit(8) {
        identification::SelectedUdmaMode { udma0: true }
    } else {
        identification::SelectedUdmaMode { unknown: true }
    };
    let supported_udma_modes = identification::SupportedUdmaModes {
        udma6: raw[88].get_bit(6),
        udma5: raw[88].get_bit(5),
        udma4: raw[88].get_bit(4),
        udma3: raw[88].get_bit(3),
        udma2: raw[88].get_bit(2),
        udma1: raw[88].get_bit(1),
        udma0: raw[88].get_bit(0),
    };
    let sata_caps = identification::SataCapabilities {
        rlde_eq_rle: raw[76].get_bit(15),
        device_aptst: raw[76].get_bit(14),
        host_aptst: raw[76].get_bit(13),
        ncq_priority_info: raw[76].get_bit(12),
        unload_while_ncq_outstanding: raw[76].get_bit(11),
        sata_phy: raw[76].get_bit(10),
        partial_slumber_pm: raw[76].get_bit(9),
        ncq: raw[76].get_bit(8),
        gen3: raw[76].get_bit(3),
        gen2: raw[76].get_bit(2),
        gen1: raw[76].get_bit(1),
        oob_management: raw[77].get_bit(9),
        power_disable_always_enabled: raw[77].get_bit(8),
        devsleep_to_reducedpwrstate: raw[77].get_bit(7),
        snd_recv_queued_cmds: raw[77].get_bit(6),
        ncq_nondata: raw[77].get_bit(5),
        ncq_streaming: raw[77].get_bit(4),
        negotiated_speed: speed,
        power_disable: raw[78].get_bit(12),
        rebuild_assist: raw[78].get_bit(11),
        dipm_ssp: raw[78].get_bit(10),
        hybrid_information: raw[78].get_bit(9),
        device_sleep: raw[78].get_bit(8),
        ncq_autosense: raw[78].get_bit(7),
        ssp: raw[78].get_bit(6),
        hardware_feature_control: raw[78].get_bit(5),
        in_order_data_delivery: raw[78].get_bit(4),
        dipm: raw[78].get_bit(3),
        dma_setup_auto_activation: raw[78].get_bit(2),
        nonzero_buffer_offsets: raw[78].get_bit(1),
    };
    let en_sata_caps = identification::EnabledSataCapabilities {
        rebuild_assist: raw[79].get_bit(11),
        power_disable: raw[79].get_bit(10),
        hybrid_information: raw[79].get_bit(9),
        device_sleep: raw[79].get_bit(8),
        aptst: raw[79].get_bit(7),
        ssp: raw[79].get_bit(6),
        hardware_feature_control: raw[79].get_bit(5),
        in_order_data_delivery: raw[79].get_bit(4),
        dipm: raw[79].get_bit(3),
        dma_setup_auto_activate: raw[79].get_bit(2),
        nonzero_buffer_offsets: raw[79].get_bit(1),
    };
    let cmd_ft_sets = identification::SupportedCommandFeatureSets {
        nop: raw[82].get_bit(14),
        read_buffer: raw[82].get_bit(13),
        write_buffer: raw[82].get_bit(12),
        hpa: raw[82].get_bit(10),
        device_reset: raw[82].get_bit(9),
        service: raw[82].get_bit(8),
        release: raw[82].get_bit(7),
        read_lookahead: raw[82].get_bit(6),
        volatile_write_cache: raw[82].get_bit(5),
        atapi: raw[82].get_bit(4),
        power_management: raw[82].get_bit(3),
        security: raw[82].get_bit(1),
        smart: raw[82].get_bit(0),
        flush_cache_ext: raw[83].get_bit(13),
        flush_cache: raw[83].get_bit(12),
        dco: raw[83].get_bit(11),
        lba48: raw[83].get_bit(10),
        aam: raw[83].get_bit(9),
        hpa_security: raw[83].get_bit(8),
        set_features_req: raw[83].get_bit(6),
        puis: raw[83].get_bit(5),
        apm: raw[83].get_bit(3),
        cfa: raw[83].get_bit(2),
        tcq: raw[83].get_bit(1),
        download_microcode: raw[83].get_bit(0),
        idle_immediate_unload: raw[84].get_bit(13),
        wwn: raw[84].get_bit(8),
        write_dma_queued_fua_ext: raw[84].get_bit(7),
        write_multiple_fua_ext: raw[84].get_bit(6),
        gpl: raw[84].get_bit(5),
        streaming: raw[84].get_bit(4),
        media_card_passthrough: raw[84].get_bit(3),
        media_sn: raw[84].get_bit(2),
        smart_self_test: raw[84].get_bit(1),
        smart_error_logging: raw[84].get_bit(0),
        freefall_control: raw[119].get_bit(5),
        download_microcode_offset: raw[119].get_bit(4),
        rw_log_dma_ext: raw[119].get_bit(3),
        write_uncorrectable_ext: raw[119].get_bit(2),
        write_read_verify: raw[119].get_bit(1),
    };
    let en_cmd_ft_sets = identification::EnabledCommandFeatureSets {
        nop: raw[85].get_bit(14),
        read_buffer: raw[85].get_bit(13),
        write_buffer: raw[85].get_bit(12),
        hpa: raw[85].get_bit(10),
        device_reset: raw[85].get_bit(9),
        service: raw[85].get_bit(8),
        release: raw[85].get_bit(7),
        read_lookahead: raw[85].get_bit(6),
        volatile_write_cache: raw[85].get_bit(5),
        atapi: raw[85].get_bit(4),
        power_management: raw[85].get_bit(3),
        security: raw[85].get_bit(1),
        smart: raw[85].get_bit(0),
        flush_cache_ext: raw[86].get_bit(13),
        flush_cache: raw[86].get_bit(12),
        dco: raw[86].get_bit(11),
        lba48: raw[86].get_bit(10),
        aam: raw[86].get_bit(9),
        hpa_security: raw[86].get_bit(8),
        set_features_req: raw[86].get_bit(6),
        puis: raw[86].get_bit(5),
        apm: raw[86].get_bit(3),
        cfa: raw[86].get_bit(2),
        tcq: raw[86].get_bit(1),
        download_microcode: raw[86].get_bit(0),
        idle_immediate_unload: raw[87].get_bit(13),
        wwn: raw[87].get_bit(8),
        write_dma_queued_fua_ext: raw[87].get_bit(7),
        write_multiple_fua_ext: raw[87].get_bit(6),
        smart_self_test: raw[87].get_bit(5),
        media_card_passthrough: raw[87].get_bit(3),
        valid_media_sn: raw[87].get_bit(2),
        smart_error_logging: raw[87].get_bit(0),
        freefall_control: raw[120].get_bit(5),
        download_microcode_offset: raw[120].get_bit(4),
        rw_log_dma_ext: raw[120].get_bit(3),
        write_uncorrectable_ext: raw[120].get_bit(2),
        write_read_verify: raw[120].get_bit(1),
    };
    let security_status = identification::SecurityStatus {
        master_pwd_cap: raw[128].get_bit(8),
        enhanced_erase: raw[128].get_bit(5),
        pwd_counter_exceeded: raw[128].get_bit(4),
        frozen: raw[128].get_bit(3),
        locked: raw[128].get_bit(2),
    };
    let cfa_power_mode = identification::CFAPowerMode {
        mode1: raw[160].get_bit(1),
        mode0: raw[160].get_bit(0),
        rms_current: raw[160].get_bits(0..12) as u16,
    };
    let sct = identification::SCTCommandTransport {
        sct_data_tables: raw[206].get_bit(5),
        sct_feature_control: raw[206].get_bit(4),
        sct_error_recovery: raw[206].get_bit(3),
        sct_write_same: raw[206].get_bit(2),
        sct_rw_long: raw[206].get_bit(1),
        sct_supported: raw[206].get_bit(0),
    };
    let nvcache_caps = identification::NVCacheCapabilities {
        nvcache_enabled: raw[214].get_bit(4),
        nvcache_pm_enabled: raw[214].get_bit(1),
        nvcache_power_supported: raw[214].get_bit(0),
    };
    let total_sectors_lba28 = {
        let (lobytes, hibytes) = (raw[60].to_le_bytes(), raw[61].to_le_bytes());
        u32::from_le_bytes([lobytes[0], lobytes[1], hibytes[0], hibytes[1]])
    };
    let total_sectors_lba48 = {
        let (lobytes, hibytes) = (
            [raw[100].to_le_bytes(), raw[101].to_le_bytes()],
            [raw[102].to_le_bytes(), raw[103].to_le_bytes()],
        );
        u64::from_le_bytes([
            lobytes[0][0],
            lobytes[0][1],
            lobytes[1][0],
            lobytes[1][1],
            hibytes[0][0],
            hibytes[0][1],
            hibytes[1][0],
            hibytes[1][1],
        ])
    };
    let stream_performance_granularity = {
        let (lobytes, hibytes) = (raw[98].to_le_bytes(), raw[99].to_le_bytes());
        u32::from_le_bytes([lobytes[0], lobytes[1], hibytes[0], hibytes[1]])
    };
    let logical_sector_size = {
        let (lobytes, hibytes) = (raw[117].to_le_bytes(), raw[118].to_le_bytes());
        u32::from_le_bytes([lobytes[0], lobytes[1], hibytes[0], hibytes[1]])
    };
    let wrv_count_mode3 = {
        let (lobytes, hibytes) = (raw[210].to_le_bytes(), raw[211].to_le_bytes());
        u32::from_le_bytes([lobytes[0], lobytes[1], hibytes[0], hibytes[1]])
    };
    let wrv_count_mode2 = {
        let (lobytes, hibytes) = (raw[212].to_le_bytes(), raw[213].to_le_bytes());
        u32::from_le_bytes([lobytes[0], lobytes[1], hibytes[0], hibytes[1]])
    };
    let nvcache_size_logical = {
        let (lobytes, hibytes) = (raw[215].to_le_bytes(), raw[216].to_le_bytes());
        u32::from_le_bytes([lobytes[0], lobytes[1], hibytes[0], hibytes[1]])
    };
    identification::DeviceIdentification {
        gen_config,
        spec_config: raw[2],
        sn,
        fw_rev,
        model_number,
        max_sectors_drq_blk: raw[47].get_bits(0..8) as u8,
        tc_supported: raw[48].get_bit(0),
        capabilities,
        logical_sectors_rw_multiple: raw[59].get_bits(0..8) as u8,
        total_sectors_lba28,
        selected_mwdma_mode,
        min_mwdma_cycle_time: raw[65],
        recommended_mwdma_cycle_time: raw[66],
        min_pio_cycle_time_no_iordy: raw[67],
        min_pio_cycle_time_iordy: raw[68],
        queue_depth: raw[75].get_bits(0..5) as u8,
        sata_caps,
        en_sata_caps,
        major: raw[80],
        minor: raw[81],
        cmd_ft_sets,
        en_cmd_ft_sets,
        current_udma_mode,
        supported_udma_modes,
        normal_sec_erasure_time: raw[89],
        enhanced_sec_erasure_time: raw[90],
        current_apm_level: raw[91].get_bits(0..8) as u8,
        master_pwd_identifier: raw[92],
        hw_test_res: raw[93],
        recommended_aam_level: raw[94].get_bits(8..16) as u8,
        current_aam_level: raw[94].get_bits(0..8) as u8,
        stream_min_req_size: raw[95],
        dma_stream_transfer_time: raw[96],
        stream_access_latency: raw[97],
        stream_performance_granularity,
        total_sectors_lba48,
        pio_stream_transfer_time: raw[104],
        logical_sectors_per_phys_sector: raw[106],
        wwn: [raw[108], raw[109], raw[110], raw[111]],
        logical_sector_size,
        security_status,
        cfa_power_mode,
        nominal_form_factor: raw[168].get_bits(0..4) as u8,
        current_media_sn,
        sct,
        logical_sector_alignment: raw[209],
        wrv_count_mode3,
        wrv_count_mode2,
        nvcache_caps,
        nvcache_size_logical,
        nominal_rotation_rate: raw[217],
        nvcache_retrieval_time: raw[219].get_bits(0..8) as u8,
        wrv_mode: raw[220].get_bits(0..7) as u8,
        transport_major: raw[222],
        transport_minor: raw[223],
        min_blocks_for_microcode: raw[234],
        max_blocks_for_microcode: raw[235],
    }
}
