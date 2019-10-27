mod internal;
extern crate alloc;
use crate::memory::*;
use crate::pci;
use crate::{printk, printkln};
use alloc::format;
use bit_field::BitField;
use core::mem::size_of;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::hlt;
use alloc::vec::Vec;

lazy_static! {
// HBADB: An array of up to 64 Host bus adapters (HBAs)
// Allows for up to 2,048 HBA ports
static ref HBADB: Mutex<[AhciDevice; 64]> = Mutex::new([AhciDevice {
bar: 0,
device: pci::PCIDevice::default(),
idx: 0,
}; 64]);
}

#[derive(Clone, Debug, Copy)]
pub struct AhciDevice {
    pub idx: usize,
    pub bar: u64,
    pub device: pci::PCIDevice,
}

// SATA/ATA signatures
const SIG_SATA: u64 = 0x00000101; // SATA drive
const SIG_ATAPI: u64 = 0xEB140101; // SATAPI drive
const SIG_SEM: u64 = 0xC33C0101; // Enclosure management bridge
const SIG_PM: u64 = 0x96690101; // Port multiplier

// Base address, 4M
const AHCI_BASE: u32 = 0x400000;

#[repr(u8)]
pub enum AhciDeviceType {
    Null = 0,
    Sata,
    Sem,
    Pm,
    Satapi,
}

#[repr(u8)]
pub enum HBAPortStatus {
    DetPresent = 3,
    IpmActive = 1,
}

#[repr(u16)]
pub enum PortCommand {
    Cr = 1 << 15,
    Fr = 1 << 14,
    Fre = 1 << 4,
    Sud = 1 << 1,
    St = 1 << 0,
}

#[repr(u8)]
pub enum AtaStatus {
    Busy = 0x80,
    Drq = 0x08,
}

#[repr(u32)]
pub enum AhciCommand {
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

#[repr(u32)]
pub enum DCOSubcommand {
    DeviceConfigurationFreezeLock = 0xC1,
    DeviceConfigurationIdentify = 0xC2,
    DeviceConfigurationRestore = 0xC0,
    DeviceConfigurationSet = 0xC3,
}

#[repr(u32)]
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
pub enum FisType {
    RegH2D = 0x27,
    RegD2H = 0x34,
    DmaAct = 0x39,
    DmaSetup = 0x41,
    Data = 0x46,
    Bist = 0x58,
    PioSetup = 0x5F,
    DevBits = 0xA1,
}

pub fn init() {
    allocate_page_range(AHCI_BASE as u64, AHCI_BASE as u64 + 100000);
    for dev in pci::get_devices() {
        if dev.class == 0x01 && dev.subclass == 0x06 && dev.prog_if == 0x01 {
            printkln!(
                "AHCI: found AHCI-capable device with vendor {:X} and device {:X}",
                dev.vendor,
                dev.device
            );
            let mut hbadb = HBADB.lock();
            let bars = match dev.header_type {
                0x00 => dev.gen_dev_tbl.unwrap().bars,
                0x01 => [
                    dev.pci_to_pci_bridge_tbl.unwrap().bars[0],
                    dev.pci_to_pci_bridge_tbl.unwrap().bars[1],
                    0,
                    0,
                    0,
                    0,
                ],
                e => panic!("Header type {} is not supported for AHCI", e),
            };
            // Figure out our MMIO BAR address
            if bars[5] != 0 && !bars[5].get_bit(0) {
                if bars[5].get_bits(1..=2) == 1 {
                    printkln!("AHCI: skipping AHCI device {:X}:{:X}: AHCI device has 16-bit BAR address {:X}", dev.vendor, dev.device, bars[5]);
                    continue;
                }
                allocate_phys_range(bars[5], bars[5] + 0x28);
                printkln!("AHCI: detected base address for AHCI driver: {:X}", bars[5]);
                let mut pos = usize::max_value();
                for (i, hba) in hbadb.iter().enumerate() {
                    if hba.bar == 0 && hba.idx == 0 {
                        pos = i;
                        printkln!("AHCI: Inserting device at position {}", i);
                        break;
                    }
                }
                if pos != usize::max_value() {
                    hbadb[pos].bar = bars[5];
                    hbadb[pos].idx = pos;
                    hbadb[pos].device = dev;
                    let mem = unsafe { &mut *(hbadb[pos].bar as *mut internal::HbaMem) };
                    printkln!("AHCI: Device scan: dbg: {:?}", mem);
                    let pi = mem.pi;
                    for i in 0..32 {
                        if pi.get_bit(i) {
                            let port = mem.ports[i];
                            let addr = format!("{:p}", &mem.ports[i]);
                            let addr = addr.trim_start_matches("0x");
                            let ssts = port.ssts;
                            let ipm = (ssts >> 8) & 0x0F;
                            let det = ipm & 0x0F;
                            if det != HBAPortStatus::DetPresent as u32
                                && ipm != HBAPortStatus::IpmActive as u32
                            {
                                continue;
                            } else if port.sig == SIG_ATAPI as u32 {
                                printkln!("AHCI: Port {}: ATAPI device found, but ATAPI devices are not supported. Skipping", i);
                            } else if port.sig == SIG_SATA as u32 {
                                printkln!("AHCI: Port {}: SATA device found", i);
                                rebase_port(u64::from_str_radix(addr, 16).unwrap(), i as u32);
                                let buffer = 0x1000u16;
                                ata_read(u64::from_str_radix(addr, 16).unwrap(), 0, 0, 1, &buffer);
                                let mut data: Vec<u8> = Vec::new();
                                for j in 0..512 {
                                let buf = buffer as *mut u8;
                                    let ptr = unsafe { buf.offset(j as isize) };
                                    data.push(unsafe { ptr.read_volatile() } as u8);
                                }
                                for j in 0..511 {
                                    if data[i] == 0x55 && data[i + 1] == 0xAA {
                                        printkln!("AHCI: port {}: found boot signature at bytes {} and {}", i, j, j + 1);
                                        break;
                                    }
                                }
                                printk!("AHCI: data: [");
                                for j in 0..512 {
                                    if j != 511 {
                                        printk!("{}, ", data[j]);
                                    } else {
                                        printk!("{}", data[j]);
                                    }
                                }
                                printkln!("]");
                            }
                        }
                    }
                } else {
                    printkln!("AHCI: error: Cannot add HBA {:X}:{:X} to the internal HBA list: HBA maximum reached.", dev.vendor, dev.device);
                    continue;
                }
            }
        }
    }
}

pub fn start_command_engine(addr: u64) {
    let mut port = unsafe { &mut *(addr as *mut internal::HbaPort) };
    while port.cmd & PortCommand::Cr as u32 == 1 {
        hlt();
    }
    port.cmd |= PortCommand::Fre as u32;
    port.cmd |= PortCommand::St as u32;
}

pub fn stop_command_engine(addr: u64) {
    let mut port = unsafe { &mut *(addr as *mut internal::HbaPort) };
    port.cmd &= !(PortCommand::St as u32);
    loop {
        hlt();
        if port.cmd & PortCommand::Fr as u32 == 1 {
            continue;
        }
        if port.cmd & PortCommand::Cr as u32 == 1 {
            continue;
        }
        break;
    }
    port.cmd &= !(PortCommand::Fre as u32);
}

pub fn rebase_port(addr: u64, new_port: u32) {
    let mut port = unsafe { &mut *(addr as *mut internal::HbaPort) };
    stop_command_engine(addr.clone());
    port.clb = AHCI_BASE + (new_port << 10) as u32;
    port.clbu = 0;
    port.fb = AHCI_BASE + (32 << 10) + (new_port << 8) as u32;
    port.fbu = 0;
    unsafe {
        let raw_header = port.clb as *mut internal::HbaCmdHeader;
        for i in 0..32 {
            let header =
                raw_header.offset(i as isize).as_mut().unwrap() as &mut internal::HbaCmdHeader;
            header.prdtl = 8;
            header.ctba = AHCI_BASE + (40 << 10) + (new_port << 13) + (i << 8) as u32;
            header.ctbau = 0;
        }
    }
    start_command_engine(addr.clone());
}

pub fn find_cmd_slot(addr: u64) -> i32 {
    let port = unsafe { &mut *(addr as *mut internal::HbaPort) };
    let mut slots = port.sact | port.ci;
    for i in 0..32 {
        if (slots & 1) == 0 {
            return i;
        }
        slots >>= 1;
    }
    printkln!("AHCI: fatal: cannot find free command slot");
    return -1;
}

pub fn ata_read(addr: u64, start_lo: u32, start_hi: u32, count: u32, buffer: &u16) -> bool {
    let mut port = unsafe { &mut *(addr as *mut internal::HbaPort) };
    port.is = u32::max_value();
    let mut cnt = count.clone() as u16;
    let mut buf = buffer.clone() as *mut u16;
    let mut spin = 0;
    let slot = find_cmd_slot(addr.clone());
    if slot == -1 {
        return false;
    }
    let header = unsafe {
        let raw_ptr = port.clb as *mut internal::HbaCmdHeader;
        raw_ptr.offset(slot as isize).as_mut().unwrap() as &mut internal::HbaCmdHeader
    };
    header.set_cfl((size_of::<internal::FisRegH2D>() / size_of::<u32>()) as u8);
    header.set_w(0);
    header.prdtl = (((cnt - 1) >> 4) + 1) as u16;
    let cmdtbl = unsafe {
        let raw_ptr = header.ctba as *mut internal::HbaCmdTbl;
        raw_ptr.as_mut().unwrap() as &mut internal::HbaCmdTbl
    };
    let mut i: usize = 0;
    for j in 0..(header.prdtl as usize) - 1 {
    unsafe {
        cmdtbl.prdt_entry[j].dba = *buf as u32;
        cmdtbl.prdt_entry[j].set_dbc(8 * 1024 - 1);
        cmdtbl.prdt_entry[j].set_i(1);
        buf = buf.add(4 * 1024).as_mut().unwrap();
        cnt -= 16;
        i = j;
        }
    }
    cmdtbl.prdt_entry[i].dba = buf as u32;
    cmdtbl.prdt_entry[i].set_dbc(((cnt as u32) << 9) - 1 as u32);
    cmdtbl.prdt_entry[i].set_i(1);
    let ptr = &mut cmdtbl.cfis;
    let cmdfis = unsafe { &mut *(ptr as *mut [u8; 64] as *mut internal::FisRegH2D) };
    cmdfis.fis_type = FisType::RegH2D as u8;
    cmdfis.set_c(1);
    cmdfis.command = AhciCommand::ReadDmaExt as u8;
    cmdfis.lba0 = start_lo as u8;
    cmdfis.lba1 = (start_lo >> 8) as u8;
    cmdfis.lba2 = (start_lo >> 16) as u8;
    cmdfis.device = 1 << 6;
    cmdfis.lba3 = (start_lo >> 24) as u8;
    cmdfis.lba4 = start_hi as u8;
    cmdfis.lba5 = (start_hi >> 8) as u8;
    cmdfis.count_lo = cnt.get_bits(0..7) as u8;
    cmdfis.count_hi = cnt.get_bits(8..15) as u8;
    while (port.tfd & (AtaStatus::Busy as u32 | AtaStatus::Drq as u32) > 0) && spin < 1000000 {
        spin += 1;
    }
    if spin == 1000000 {
        panic!("Detected port hang: {:?}", port);
    }
    port.ci = 1 << slot;
    loop {
        hlt();
        if port.ci & (1 << slot) == 0 {
            break;
        }
        if port.is & (1 << 30) == 1 {
            panic!("Read error with HBA port: {:?}", port);
        }
    }
    if port.is & (1 << 30) == 1 {
        panic!("Read error with HBA port: {:?}", port);
    }
    return true;
}
