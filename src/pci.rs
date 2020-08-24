// SPDX-License-Identifier: MPL-2.0
use crate::acpi;
use alloc::collections::LinkedList;
use bit_field::BitField;
use lazy_static::lazy_static;
use log::*;
use spin::RwLock;

const MAX_FUNCTION: usize = 8;
const MAX_DEVICE: usize = 32;
const MAX_BUS: usize = 256;
const MAX_SG: usize = 65536;

pub const VENDOR_ID: u32 = 0x00;
pub const DEVICE_ID: u32 = 0x02;
pub const COMMAND: u32 = 0x04;
pub const STATUS: u32 = 0x06;
pub const REV_ID: u32 = 0x08;
pub const PROG_IF: u32 = 0x09;
pub const DEV_SUBCLASS: u32 = 0x0A;
pub const DEV_CLASS: u32 = 0x0B;
pub const CACHE_LINE_SIZE: u32 = 0x0C;
pub const LATENCY_TIMER: u32 = 0x0D;
pub const HEADER_TYPE: u32 = 0x0E;
pub const BIST: u32 = 0x0F;
pub const BAR0: u32 = 0x10;
pub const BAR1: u32 = 0x14;
pub const BAR2: u32 = 0x18;
pub const BAR3: u32 = 0x1C;
pub const BAR4: u32 = 0x20;
pub const BAR5: u32 = 0x24;
pub const CARDBUS_CIS: u32 = 0x28;
pub const SS_VENDOR_ID: u32 = 0x2C;
pub const SS_ID: u32 = 0x2E;
pub const ROM_ADDR: u32 = 0x30;
pub const CAP_LIST: u32 = 0x34;
pub const INT_LINE: u32 = 0x3C;
pub const INT_PIN: u32 = 0x3D;
pub const MIN_GNT: u32 = 0x3E;
pub const MAX_LAT: u32 = 0x3F;
pub const PRIM_BUS: u32 = 0x18;
pub const SEC_BUS: u32 = 0x19;
pub const SUB_BUS: u32 = 0x1A;
pub const SEC_LAT_TMR: u32 = 0x1B;
pub const IO_BASE: u32 = 0x1C;
pub const IO_LIMIT: u32 = 0x1D;
pub const SEC_STATUS: u32 = 0x1E;
pub const MEM_BASE: u32 = 0x20;
pub const MEM_LMT: u32 = 0x22;
pub const PREF_MEM_BASE: u32 = 0x24;
pub const PREF_MEM_LMT: u32 = 0x26;
pub const PREF_MEM_BASE_UPPER32: u32 = 0x28;
pub const PREF_MEM_LMT_UPPER32: u32 = 0x2C;
pub const IO_BASE_UPPER16: u32 = 0x30;
pub const IO_LMT_UPPER32: u32 = 0x32;
pub const ROM_ADDR1: u32 = 0x38;
pub const BRIDGE_CTL: u32 = 0x3E;
pub const CB_CAP_LST: u32 = 0x14;
pub const CB_SEC_STATUS: u32 = 0x16;
pub const CB_PRIM_BUS: u32 = 0x18;
pub const CB_CARD_BUS: u32 = 0x19;
pub const CB_SUB_BUS: u32 = 0x1A;
pub const CB_LAT_TMR: u32 = 0x1B;
pub const CB_MEMBASE0: u32 = 0x1C;
pub const CB_MEMLMT0: u32 = 0x20;
pub const CB_MEMBASE1: u32 = 0x24;
pub const CB_MEMLMT1: u32 = 0x28;
pub const CB_IO_BASE0: u32 = 0x2C;
pub const CB_IO_BASE0_HI: u32 = 0x2E;
pub const CB_IO_LMT0: u32 = 0x30;
pub const CB_IO_LMT0_HI: u32 = 0x32;
pub const CB_IO_BASE1: u32 = 0x34;
pub const CB_IO_BASE1_HI: u32 = 0x36;
pub const CB_IO_LMT1: u32 = 0x38;
pub const CB_IO_LMT1_HI: u32 = 0x3A;
pub const CB_BR_CTL: u32 = 0x3E;
pub const CB_SS_VNDR_ID: u32 = 0x40;
pub const CB_SS_ID: u32 = 0x42;
pub const CB_LEG_MODE_BASE: u32 = 0x44;

lazy_static! {
    static ref PCI_DEVICES: RwLock<LinkedList<PCIDevice>> = RwLock::new(LinkedList::new());
}

/// Contains PCI device properties.
#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct PCIDevice {
    pub segment_group: u16,
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    pub vendor: u16,
    pub device: u16,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub header_type: u8,
    pub secondary_bus: u8,
    pub phys_addr: u64,
}

// Adds a device to the PCI device list.
fn add_device(device: PCIDevice) {
    let mut devices = PCI_DEVICES.write();
    devices.push_back(device);
}

#[no_mangle]
pub extern "C" fn read_dword(phys_addr: usize, addr: u32) -> u32 {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u32> = unsafe { VolAddress::new(phys_addr).offset(addr as isize) };
    cfgspace.read()
}

#[no_mangle]
pub extern "C" fn read_word(phys_addr: usize, addr: u32) -> u16 {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u16> = unsafe { VolAddress::new(phys_addr).offset(addr as isize) };
    cfgspace.read()
}

#[no_mangle]
pub extern "C" fn read_byte(phys_addr: usize, addr: u32) -> u8 {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u8> = unsafe { VolAddress::new(phys_addr).offset(addr as isize) };
    cfgspace.read()
}

#[no_mangle]
pub extern "C" fn write_dword(phys_addr: usize, addr: u32, value: u32) {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u32> = unsafe { VolAddress::new(phys_addr).offset(addr as isize) };
    cfgspace.write(value);
}

#[no_mangle]
pub extern "C" fn write_word(phys_addr: usize, addr: u32, value: u16) {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u16> = unsafe { VolAddress::new(phys_addr).offset(addr as isize) };
    cfgspace.write(value);
}

#[no_mangle]
pub extern "C" fn write_byte(phys_addr: usize, addr: u32, value: u8) {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u8> = unsafe { VolAddress::new(phys_addr).offset(addr as isize) };
    cfgspace.write(value);
}

pub async fn probe() {
    if let Ok(table) = acpi::init() {
        if let Some(regions) = table.pci_config_regions {
            for bus in 0..MAX_BUS {
                for device in 0..MAX_DEVICE {
                    for function in 0..MAX_FUNCTION {
                        if let Some(addr) =
                            regions.physical_address(0, bus as u8, device as u8, function as u8)
                        {
                            use crate::memory::{allocate_phys_range, free_range};
                            allocate_phys_range(addr, addr + 4096);
                            let mut dev = PCIDevice::default();
                            dev.segment_group = 0;
                            dev.bus = bus as u8;
                            dev.slot = device as u8;
                            dev.function = function as u8;
                            dev.phys_addr = addr;
                            let vendev = read_dword(addr as usize, VENDOR_ID);
                            dev.vendor = (vendev & 0xffff) as u16;
                            dev.device = (vendev >> 16) as u16;
                            if dev.vendor == 0xFFFF {
                                free_range(addr, addr + 4096);
                                drop(dev);
                                continue;
                            }
                            dev.class = read_byte(addr as usize, DEV_CLASS);
                            dev.subclass = read_byte(addr as usize, DEV_SUBCLASS);
                            dev.prog_if = read_byte(addr as usize, PROG_IF);
                            dev.revision = read_byte(addr as usize, REV_ID);
                            dev.header_type = read_byte(addr as usize, HEADER_TYPE);
                            let v = dev.header_type & 0x7F;
                            if v == 1 || v == 2 {
                                // Bridge or PCI card bus
                                let secbus = read_byte(addr as usize, SEC_BUS);
                                dev.secondary_bus = secbus;
                            }
                            info!("PCI device at addr {:X} (vd={:X}:{:X} c={:X} sc={:X} pi={:X} rev={:X} bi=0:{:X}:{:X}:{:X}) found", addr, dev.vendor, dev.device, dev.class, dev.subclass, dev.prog_if, dev.revision, bus, device, function);
                            add_device(dev);
                        } else {
                            continue;
                        }
                    }
                }
            }
        } else {
            error!("No PCI regions");
        }
    } else {
        error!("ACPI unsupported");
    }
}

pub async fn init() {
    probe().await;
}

fn calculate_bar_addr(dev: &PCIDevice, addr: u32) -> usize {
    let bar1 = read_dword(dev.phys_addr as usize, addr);
    if !bar1.get_bit(0) {
        match bar1.get_bits(1..=2) {
            0 => (bar1 & 0xFFFF_FFF0) as usize,
            1 => (bar1 & 0xFFF0) as usize,
            2 => {
                let bar2 = read_dword(
                    dev.phys_addr as usize,
                    match addr {
                        BAR0 => BAR1,
                        BAR1 => BAR2,
                        BAR2 => BAR3,
                        BAR3 => BAR4,
                        BAR4 => BAR5,
                        BAR5 | _ => 0,
                    },
                );
                (((bar1 as u64) & 0xFFFF_FFF0) + (((bar2 as u64) & 0xFFFF_FFFF) << 32)) as usize
            }
            _ => bar1 as usize,
        }
    } else {
        (bar1 & 0xFFFF_FFFC) as usize
    }
}

pub async fn find_device(class: u8, subclass: u8, interface: u8) -> Option<PCIDevice> {
    let devices = PCI_DEVICES.read();
    for dev in devices.iter() {
        if dev.class == class && dev.subclass == subclass && dev.prog_if == interface {
            return Some(*dev);
        }
    }
    None
}

pub async fn get_bar(idx: u8, class: u8, subclass: u8, interface: u8) -> Option<usize> {
    if let Some(dev) = find_device(class, subclass, interface).await {
        match idx {
            0 => return Some(calculate_bar_addr(&dev, BAR0)),
            1 => return Some(calculate_bar_addr(&dev, BAR1)),
            2 => return Some(calculate_bar_addr(&dev, BAR2)),
            3 => return Some(calculate_bar_addr(&dev, BAR3)),
            4 => return Some(calculate_bar_addr(&dev, BAR4)),
            5 => return Some(calculate_bar_addr(&dev, BAR5)),
            _ => return None,
        }
    } else {
        return None;
    }
}
