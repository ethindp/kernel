// SPDX-License-Identifier: MPL-2.0
use bit_field::BitField;
use hashbrown::HashMap;
use lazy_static::lazy_static;
use log::*;
use minivec::MiniVec;
use spin::RwLock;
use voladdress::*;
use x86::random;

const MAX_FUNCTION: usize = 8;
const MAX_DEVICE: usize = 32;
const MAX_BUS: usize = 256;
const MAX_SG: usize = 65536;

const VENDOR_ID: u32 = 0x00;
const DEVICE_ID: u32 = 0x02;
const COMMAND: u32 = 0x04;
const STATUS: u32 = 0x06;
const REV_ID: u32 = 0x08;
const PROG_IF: u32 = 0x09;
const DEV_SUBCLASS: u32 = 0x0A;
const DEV_CLASS: u32 = 0x0B;
const HEADER_TYPE: u32 = 0x0E;
const BIST: u32 = 0x0F;
const BAR0: u32 = 0x10;
const BAR1: u32 = 0x14;
const BAR2: u32 = 0x18;
const BAR3: u32 = 0x1C;
const BAR4: u32 = 0x20;
const BAR5: u32 = 0x24;
const CARDBUS_CIS: u32 = 0x28;
const SS_VENDOR_ID: u32 = 0x2C;
const SS_ID: u32 = 0x2E;
const ROM_ADDR: u32 = 0x30;
const CAP_LIST: u32 = 0x34;
const INT_LINE: u32 = 0x3C;
const INT_PIN: u32 = 0x3D;
const MIN_GNT: u32 = 0x3E;
const SEC_BUS: u32 = 0x19;
const SUB_BUS: u32 = 0x1A;
const IO_BASE: u32 = 0x1C;
const IO_LIMIT: u32 = 0x1D;
const SEC_STATUS: u32 = 0x1E;
const MEM_BASE: u32 = 0x20;
const MEM_LMT: u32 = 0x22;
const PREF_MEM_BASE: u32 = 0x24;
const PREF_MEM_LMT: u32 = 0x26;
const PREF_MEM_BASE_UPPER32: u32 = 0x28;
const PREF_MEM_LMT_UPPER32: u32 = 0x2C;
const IO_BASE_UPPER16: u32 = 0x30;
const IO_LMT_UPPER32: u32 = 0x32;
const ROM_ADDR1: u32 = 0x38;
const BRIDGE_CTL: u32 = 0x3E;
const CB_CAP_LST: u32 = 0x14;
const CB_SEC_STATUS: u32 = 0x16;
const CB_PRIM_BUS: u32 = 0x18;
const CB_CARD_BUS: u32 = 0x19;
const CB_SUB_BUS: u32 = 0x1A;
const CB_LAT_TMR: u32 = 0x1B;
const CB_MEMBASE0: u32 = 0x1C;
const CB_MEMLMT0: u32 = 0x20;
const CB_MEMBASE1: u32 = 0x24;
const CB_MEMLMT1: u32 = 0x28;
const CB_IO_BASE0: u32 = 0x2C;
const CB_IO_BASE0_HI: u32 = 0x2E;
const CB_IO_LMT0: u32 = 0x30;
const CB_IO_LMT0_HI: u32 = 0x32;
const CB_IO_BASE1: u32 = 0x34;
const CB_IO_BASE1_HI: u32 = 0x36;
const CB_IO_LMT1: u32 = 0x38;
const CB_IO_LMT1_HI: u32 = 0x3A;
const CB_BR_CTL: u32 = 0x3E;
const CB_SS_VNDR_ID: u32 = 0x40;
const CB_SS_ID: u32 = 0x42;
const CB_LEG_MODE_BASE: u32 = 0x44;

type DeviceInformation = (u8, u8, u8); // Class, subclass, program interface
type DeviceIdentification = MiniVec<DeviceInformation>; // Device information, optional alteration of int line.
type InitFunc = fn(PCIDevice);

lazy_static! {
    static ref PCI_DEVICES: RwLock<MiniVec<PCIDevice>> = RwLock::new(MiniVec::new());
    static ref DEV_INIT_FUNCS: RwLock<HashMap<DeviceIdentification, InitFunc>> = RwLock::new({
        let mut map: HashMap<DeviceIdentification, InitFunc> = HashMap::new();
        if cfg!(feature = "nvme") {
            use minivec::mini_vec;
            let dinfo = mini_vec![(0x01, 0x08, 0x02), (0x01, 0x08, 0x03)];
            map.insert(dinfo, crate::nvme::init);
        }
        map
    });
}

/// Contains PCI device properties.
/// This structure contains only static properties that remain unchanged.
#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct PCIDevice {
/// Segment group of the device. There can be up to 65536 segment groups.
    pub segment_group: u16,
    /// Bus number of the device.
    pub bus: u8,
    /// Slot number of the device.
    pub slot: u8,
    /// Function number of the device. A multifunction device is free to implement multiple
    /// functions, so a different function does not always mean that this is a different
    /// device.
    /// However, the system treats each (sg, bus, slot, function) combination as a new device.
    pub function: u8,
    /// Vendor ID of this device. Assigned by PCI-SIG.
    pub vendor: u16,
    /// Device ID. Assigned by the vendor.
    pub device: u16,
    /// Class of this device.
    pub class: u8,
    /// Subclass of this device.
    pub subclass: u8,
    /// Program interface of this device.
    pub prog_if: u8,
    /// Revision ID of this device.
    pub revision: u8,
    /// Secondary bus register.
    pub secondary_bus: u8,
    /// Physical memory address in PCIe configuration space for this device.
    pub phys_addr: u64,
    /// Base address registers (BARs) for this device. Not all BARs need to be implemented.
    /// BARs that are 0 are unimplemented and should not be accessed by device drivers.
    pub bars: (u64, u64, u64, u64, u64, u64),
    /// Header type; encodes the data beyond offset 10h in PCIe configuration space. A value
    /// of 00h indicates a normal device. A value of 01h indicates a PCI-to-PCI bridge,
    /// which is documented in the PCI to PCI Bridge Architecture Specification. A value of
    /// 02h indicates a CardBus bridge and is specified in the PC Card Standard.
    pub htype: u8,
    /// Card information structure (CIS) CardBus pointer.
    pub cis_ptr: u32,
    /// Subsystem ID.
    pub ssid: u16,
    /// Subsystem vendor ID.
    pub ssvid: u16,
    /// Expansion ROM base address
    pub exp_rom_base_addr: u32,
    /// Capabilities pointer
    pub caps_ptr: u8,
    /// Interrupt pin number
    pub int_pin: u8,
    /// Interrupt line (not used for MSI or MSI-X). This value is altered if the function
    /// supports MSI or MSI-X and contains the randomly generated interrupt value.
    pub int_line: u8,
    /// Contains a unique device ID. Device drivers may use this (e.g.: to allow interrupt->driver communication).
    pub unique_dev_id: u128,
}

// Adds a device to the PCI device list.
fn add_device(device: PCIDevice) {
    let mut devs = PCI_DEVICES.write();
    let l = devs.len();
    devs.reserve(l * 2);
    devs.push(device);
}

fn read_dword(phys_addr: usize, addr: u32) -> u32 {
    let cfgspace: VolAddress<u32> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.read()
}

fn read_word(phys_addr: usize, addr: u32) -> u16 {
    let cfgspace: VolAddress<u16> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.read()
}

fn read_byte(phys_addr: usize, addr: u32) -> u8 {
    let cfgspace: VolAddress<u8> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.read()
}

fn write_dword(phys_addr: usize, addr: u32, value: u32) {
    let cfgspace: VolAddress<u32> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.write(value);
}

fn write_word(phys_addr: usize, addr: u32, value: u16) {
    let cfgspace: VolAddress<u16> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.write(value);
}

fn write_byte(phys_addr: usize, addr: u32, value: u8) {
    let cfgspace: VolAddress<u8> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.write(value);
}

/// Probes the PCI bus.
#[cold]
pub async fn probe() {
    if let Ok(regions) = crate::acpi::get_pci_regions() {
        info!("Scanning segment groups");
        let mut maxsg: u16 = 0;
        (0..MAX_SG).for_each(|i| {
            if regions.physical_address(i as u16, 0, 0, 0).is_some() {
                maxsg += 1;
            }
        });
        info!(
            "{} {} found",
            maxsg,
            if maxsg > 1 {
                "segment groups"
            } else {
                "segment group"
            }
        );
        (0u16..maxsg).for_each(|sg|
        (0 .. MAX_BUS).for_each(|bus|
        (0 .. MAX_DEVICE).for_each(|device|
        (0 .. MAX_FUNCTION).for_each(|function| {
                        if let Some(addr) =
                            regions.physical_address(sg, bus as u8, device as u8, function as u8)
                        {
                            use crate::memory::{allocate_phys_range, free_range};
                            let _ = allocate_phys_range(addr, addr + 4096, true);
                            if (read_dword(addr as usize, VENDOR_ID) & 0xFFFF) == 0xFFFF {
                                free_range(addr, addr + 4096);
                                return;
}
                            let mut dev = PCIDevice {
                            segment_group: sg,
                            bus: bus as u8,
                            slot: device as u8,
                            function: function as u8,
                            phys_addr: addr,
                            vendor: (read_dword(addr as usize, VENDOR_ID) & 0xffff) as u16,
                            device: (read_dword(addr as usize, VENDOR_ID) >> 16) as u16,
                            class: read_byte(addr as usize, DEV_CLASS),
                            subclass: read_byte(addr as usize, DEV_SUBCLASS),
                            prog_if: read_byte(addr as usize, PROG_IF),
                            revision: read_byte(addr as usize, REV_ID),
                            htype: read_byte(addr as usize, HEADER_TYPE) & 0x7F,
                                // Bridge or PCI card bus
                                secondary_bus:                             if (read_byte(addr as usize, HEADER_TYPE) & 0x7F) == 1 || (read_byte(addr as usize, HEADER_TYPE) & 0x7F) == 2 {
                                read_byte(addr as usize, SEC_BUS)
                            } else {
                            0
                            },
                            bars: (0, 0, 0, 0, 0, 0),
                            cis_ptr: read_dword(addr as usize, CARDBUS_CIS),
                            ssid: read_word(addr as usize, SS_ID),
                            ssvid: read_word(addr as usize, SS_VENDOR_ID),
                            exp_rom_base_addr: read_dword(addr as usize, ROM_ADDR),
                            caps_ptr: read_byte(addr as usize, CAP_LIST),
                            int_pin: read_byte(addr as usize, INT_PIN),
                            int_line: read_byte(addr as usize, INT_LINE),
                            unique_dev_id: 0,
                            };
                            dev.bars = (calculate_bar_addr(&dev, BAR0) as u64, calculate_bar_addr(&dev, BAR1) as u64, calculate_bar_addr(&dev, BAR2) as u64, calculate_bar_addr(&dev, BAR3) as u64, calculate_bar_addr(&dev, BAR4) as u64, calculate_bar_addr(&dev, BAR5) as u64);
                            let mut dev_id = 0u128;
                            dev_id.set_bits(0 .. 16, dev.segment_group as u128);
                            dev_id.set_bits(16 .. 24, dev.bus as u128);
                            dev_id.set_bits(24 .. 32, dev.slot as u128);
                            dev_id.set_bits(32 .. 40, dev.function as u128);
                            dev_id.set_bits(40 .. 56, dev.vendor as u128);
                            dev_id.set_bits(56 .. 72, dev.device as u128);
                            dev_id.set_bits(72 .. 88, dev.ssid as u128);
                            dev_id.set_bits(88 .. 104, dev.ssvid as u128);
                            let mut random_bits: u64 = 0;
                            unsafe {
                            random::rdrand64(&mut random_bits);
                            }
                            random_bits = random_bits.wrapping_mul(9908962810164294844);
                            dev_id.set_bits(104 .. 128, random_bits.get_bits(0 .. 24) as u128);
                            dev.unique_dev_id = dev_id;
                            let mut cmd = read_word(addr as usize, COMMAND);
                            cmd.set_bit(0, true);
                            cmd.set_bit(1, true);
                            cmd.set_bit(2, true);
                            cmd.set_bit(3, false);
                            cmd.set_bit(4, false);
                            cmd.set_bit(5, false);
                            cmd.set_bit(6, true);
                            cmd.set_bit(7, false);
                            cmd.set_bit(8, true);
                            cmd.set_bit(9, false);
                            cmd.set_bit(10, true);
                            cmd.set_bits(11 .. 16, 0);
                            write_word(addr as usize, COMMAND, cmd);
                            info!("Detected device of type {} with vendor ID of {:X} and subsystem ID {:X}", classify_program_interface(dev.class, dev.subclass, dev.prog_if).unwrap_or_else(|| classify_subclass(dev.class, dev.subclass).unwrap_or_else(|| classify_class(dev.class).unwrap_or("Unknown Device"))), dev.vendor, dev.ssid);
                            if read_word(addr as usize, STATUS).get_bit(4) {
                            info!("Device implements capabilities list, scanning");
                            let mut caddr = (addr as usize) + (dev.caps_ptr as usize);
                            loop {
                            let caps = read_dword(caddr, 0x00);
                            let id = caps.get_bits(0 .. 8);
                            let nptr = caps.get_bits(8 .. 16);
                            if nptr == 0x00 || nptr == 0xff {
                            break;
                            }
                            info!("Discovered capability ID {:X}h at addr {:X}h", id, caddr);
                            if id == 0x11 {
                            info!("Device supports MSI-X, enabling");
                            let mut int: u8 = 0;
                            // The algorithm for trying to re-randomize a 16-bit number comes from the 6502 forums: http://forum.6502.org/viewtopic.php?f=2&t=5247&sid=01f33d4f5663073b3bd1abcc62cdffb8&start=0
                            while int < 0x30 && int != 0xFE && int != 0xFD {
                            let mut i = 0u16;
                            unsafe {
                            random::rdrand16(&mut i);
                            }
                            i = i.wrapping_mul(0x7ABD).wrapping_add(0x1B0F) % 0xFC;
                            int = i.get_bits(0 .. 8) as u8;
                            }
                            dev.int_line = int;
                            info!("Using interrupt {} ({:X}h)", int, int);
                            let mc = read_dword(caddr, 0x00);
                            let tsize = mc.get_bits(16 .. 27) + 1;
                            let mc = read_dword(caddr, 0x04);
                            let memstart = match mc.get_bits(0 .. 3) {
                            0 => dev.bars.0 + (mc.get_bits(3 .. 32) as u64),
                            1 => dev.bars.1 + (mc.get_bits(3 .. 32) as u64),
                            2 => dev.bars.2 + (mc.get_bits(3 .. 32) as u64),
                            3 => dev.bars.3 + (mc.get_bits(3 .. 32) as u64),
                            4 => dev.bars.4 + (mc.get_bits(3 .. 32) as u64),
                            5 => dev.bars.5 + (mc.get_bits(3 .. 32) as u64),
                            e => panic!("Device uses BAR {:X}h for MSI-X, which is not valid!", e)
                            };
                            info!("Using memaddr {:X}h for MSI-X, table size: {:X}h", memstart, tsize);
                            allocate_phys_range(memstart, memstart + (16 * (tsize as u64)), true);
let table: DynamicVolBlock<u128> = unsafe { DynamicVolBlock::new(memstart as usize, tsize as usize) };
(0 .. (tsize-1) as usize).for_each(|e| {
let mut entry = table.index(e).read();
entry.set_bit(96, false);
let mut msgaddr = 0u64;
msgaddr.set_bits(32 .. 64, 0);
msgaddr.set_bits(20 .. 32, 0x0FEE);
msgaddr.set_bits(12 .. 20, 0xFF);
msgaddr.set_bit(3, false);
msgaddr.set_bit(2, false);
msgaddr.set_bits(0 .. 2, 0);
entry.set_bits(0 .. 64, msgaddr as u128);
let mut msgdata = 0u32;
msgdata.set_bit(15, false);
msgdata.set_bit(14, false);
msgdata.set_bits(8 .. 11, 0x00);
msgdata.set_bits(0 .. 8, int as u32);
msgdata.set_bits(16 .. 32, 0);
entry.set_bits(64 .. 96, msgdata as u128);
debug!("Vector {}: vector control={:X}, message data={:X}, message address={:X}, vector entry={:X}", e, entry.get_bits(96 .. 128), entry.get_bits(64 .. 96), entry.get_bits(0 .. 64), entry);
table.index(e).write(entry);
});
let mut mc = read_dword(caddr, 0x00);
mc.set_bit(31, true);
write_dword(caddr, 0x00, mc);
                            }
                            caddr += read_word(caddr, 0x0).get_bits(8 .. 16) as usize;
                            }
                            }
                            let funcs = DEV_INIT_FUNCS.read();
                            funcs.iter().filter(|(k, _)| {
                            let devs: MiniVec<&DeviceInformation> = k.iter().filter(|dinfo| read_byte(addr as usize, DEV_CLASS) == dinfo.0 && read_byte(addr as usize, DEV_SUBCLASS) == dinfo.1 && read_byte(addr as usize, PROG_IF) == dinfo.2).collect();
                            !devs.is_empty()
                            }).for_each(|(_, v)| {
                            info!("Found device driver for class={:X}, subclass={:X}, program interface={:X}; initializing", dev.class, dev.subclass, dev.prog_if);
                            (v)(dev);
                            });
                            add_device(dev);
                        }
                    }))));
        let mut devs = PCI_DEVICES.write();
        devs.shrink_to_fit();
    } else {
        error!("No PCI regions");
    }
}

/// Initializes the PCI subsystem.
#[cold]
pub async fn init() {
    info!("Initiating PCE bus scan");
    probe().await;
    info!(
        "PCIe scan complete; {} devices found",
        PCI_DEVICES.read().len()
    );
}

#[inline]
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
                        _ => 0,
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

/// Locates a PCI device using a class, subclass and program interface.
pub async fn find_device(class: u8, subclass: u8, interface: u8) -> Option<PCIDevice> {
    let devs = PCI_DEVICES.read();
    devs.iter()
        .filter(|d| d.class == class && d.subclass == subclass && d.prog_if == interface)
        .cloned()
        .next()
}

#[inline]
fn classify_class(class: u8) -> Option<&'static str> {
    match class {
        0x00 => Some("Unclassified device"),
        0x01 => Some("Mass storage controller"),
        0x02 => Some("Network controller"),
        0x03 => Some("Display controller"),
        0x04 => Some("Multimedia controller"),
        0x05 => Some("Memory controller"),
        0x06 => Some("Bridge"),
        0x07 => Some("Communication controller"),
        0x08 => Some("Generic system peripheral"),
        0x09 => Some("Input device controller"),
        0x0a => Some("Docking station"),
        0x0b => Some("Processor"),
        0x0c => Some("Serial bus controller"),
        0x0d => Some("Wireless controller"),
        0x0e => Some("Intelligent controller"),
        0x0f => Some("Satellite communications controller"),
        0x10 => Some("Encryption controller"),
        0x11 => Some("Signal processing controller"),
        0x12 => Some("Processing accelerators"),
        0x13 => Some("Non-Essential Instrumentation"),
        0x40 => Some("Coprocessor"),
        _ => None,
    }
}

#[inline]
fn classify_subclass(class: u8, subclass: u8) -> Option<&'static str> {
    match (class, subclass) {
        (0x00, 0x00) => Some("Non-VGA unclassified device"),
        (0x00, 0x01) => Some("VGA compatible unclassified device"),
        (0x00, 0x05) => Some("Image coprocessor"),
        (0x01, 0x00) => Some("SCSI storage controller"),
        (0x01, 0x01) => Some("IDE interface"),
        (0x01, 0x02) => Some("Floppy disk controller"),
        (0x01, 0x03) => Some("IPI bus controller"),
        (0x01, 0x04) => Some("RAID bus controller"),
        (0x01, 0x05) => Some("ATA controller"),
        (0x01, 0x06) => Some("SATA controller"),
        (0x01, 0x07) => Some("Serial Attached SCSI controller"),
        (0x01, 0x08) => Some("Non-Volatile memory controller"),
        (0x01, 0x80) => Some("Mass storage controller"),
        (0x02, 0x00) => Some("Ethernet controller"),
        (0x02, 0x01) => Some("Token ring network controller"),
        (0x02, 0x02) => Some("FDDI network controller"),
        (0x02, 0x03) => Some("ATM network controller"),
        (0x02, 0x04) => Some("ISDN controller"),
        (0x02, 0x05) => Some("WorldFip controller"),
        (0x02, 0x06) => Some("PICMG controller"),
        (0x02, 0x07) => Some("Infiniband controller"),
        (0x02, 0x08) => Some("Fabric controller"),
        (0x02, 0x80) => Some("Network controller"),
        (0x03, 0x00) => Some("VGA compatible controller"),
        (0x03, 0x01) => Some("XGA compatible controller"),
        (0x03, 0x02) => Some("3D controller"),
        (0x03, 0x80) => Some("Display controller"),
        (0x04, 0x00) => Some("Multimedia video controller"),
        (0x04, 0x01) => Some("Multimedia audio controller"),
        (0x04, 0x02) => Some("Computer telephony device"),
        (0x04, 0x03) => Some("Audio device"),
        (0x04, 0x80) => Some("Multimedia controller"),
        (0x05, 0x00) => Some("RAM memory"),
        (0x05, 0x01) => Some("FLASH memory"),
        (0x05, 0x80) => Some("Memory controller"),
        (0x06, 0x00) => Some("Host bridge"),
        (0x06, 0x01) => Some("ISA bridge"),
        (0x06, 0x02) => Some("EISA bridge"),
        (0x06, 0x03) => Some("MicroChannel bridge"),
        (0x06, 0x04) => Some("PCI bridge"),
        (0x06, 0x05) => Some("PCMCIA bridge"),
        (0x06, 0x06) => Some("NuBus bridge"),
        (0x06, 0x07) => Some("CardBus bridge"),
        (0x06, 0x08) => Some("RACEway bridge"),
        (0x06, 0x09) => Some("Semi-transparent PCI-to-PCI bridge"),
        (0x06, 0x0a) => Some("InfiniBand to PCI host bridge"),
        (0x06, 0x80) => Some("Bridge"),
        (0x07, 0x00) => Some("Serial controller"),
        (0x07, 0x01) => Some("Parallel controller"),
        (0x07, 0x02) => Some("Multiport serial controller"),
        (0x07, 0x03) => Some("Modem"),
        (0x07, 0x04) => Some("GPIB controller"),
        (0x07, 0x05) => Some("Smard Card controller"),
        (0x07, 0x80) => Some("Communication controller"),
        (0x08, 0x00) => Some("PIC"),
        (0x08, 0x01) => Some("DMA controller"),
        (0x08, 0x02) => Some("Timer"),
        (0x08, 0x03) => Some("RTC"),
        (0x08, 0x04) => Some("PCI Hot-plug controller"),
        (0x08, 0x05) => Some("SD Host controller"),
        (0x08, 0x06) => Some("IOMMU"),
        (0x08, 0x80) => Some("System peripheral"),
        (0x08, 0x99) => Some("Timing Card"),
        (0x09, 0x00) => Some("Keyboard controller"),
        (0x09, 0x01) => Some("Digitizer Pen"),
        (0x09, 0x02) => Some("Mouse controller"),
        (0x09, 0x03) => Some("Scanner controller"),
        (0x09, 0x04) => Some("Gameport controller"),
        (0x09, 0x80) => Some("Input device controller"),
        (0x0a, 0x00) => Some("Generic Docking Station"),
        (0x0a, 0x80) => Some("Docking Station"),
        (0x0b, 0x00) => Some("386"),
        (0x0b, 0x01) => Some("486"),
        (0x0b, 0x02) => Some("Pentium"),
        (0x0b, 0x10) => Some("Alpha"),
        (0x0b, 0x20) => Some("Power PC"),
        (0x0b, 0x30) => Some("MIPS"),
        (0x0b, 0x40) => Some("Co-processor"),
        (0x0c, 0x00) => Some("FireWire (IEEE 1394)"),
        (0x0c, 0x01) => Some("ACCESS Bus"),
        (0x0c, 0x02) => Some("SSA"),
        (0x0c, 0x03) => Some("USB controller"),
        (0x0c, 0x04) => Some("Fibre Channel"),
        (0x0c, 0x05) => Some("SMBus"),
        (0x0c, 0x06) => Some("InfiniBand"),
        (0x0c, 0x07) => Some("IPMI Interface"),
        (0x0c, 0x08) => Some("SERCOS interface"),
        (0x0c, 0x09) => Some("CANBUS"),
        (0x0d, 0x00) => Some("IRDA controller"),
        (0x0d, 0x01) => Some("Consumer IR controller"),
        (0x0d, 0x10) => Some("RF controller"),
        (0x0d, 0x11) => Some("Bluetooth"),
        (0x0d, 0x12) => Some("Broadband"),
        (0x0d, 0x20) => Some("802.1a controller"),
        (0x0d, 0x21) => Some("802.1b controller"),
        (0x0d, 0x80) => Some("Wireless controller"),
        (0x0e, 0x00) => Some("I2O"),
        (0x0f, 0x01) => Some("Satellite TV controller"),
        (0x0f, 0x02) => Some("Satellite audio communication controller"),
        (0x0f, 0x03) => Some("Satellite voice communication controller"),
        (0x0f, 0x04) => Some("Satellite data communication controller"),
        (0x10, 0x00) => Some("Network and computing encryption device"),
        (0x10, 0x10) => Some("Entertainment encryption device"),
        (0x10, 0x80) => Some("Encryption controller"),
        (0x11, 0x00) => Some("DPIO module"),
        (0x11, 0x01) => Some("Performance counters"),
        (0x11, 0x10) => Some("Communication synchronizer"),
        (0x11, 0x20) => Some("Signal processing management"),
        (0x11, 0x80) => Some("Signal processing controller"),
        (0x12, 0x00) => Some("Processing accelerators"),
        (0x12, 0x01) => Some("AI Inference Accelerator"),
        (_, _) => None,
    }
}

#[inline]
fn classify_program_interface(class: u8, subclass: u8, pi: u8) -> Option<&'static str> {
    match (class, subclass, pi) {
(0x01, 0x01, 0x00) => Some("ISA Compatibility mode-only controller"),
(0x01, 0x01, 0x05) => Some("PCI native mode-only controller"),
(0x01, 0x01, 0x0a) => Some("ISA Compatibility mode controller, supports both channels switched to PCI native mode"),
(0x01, 0x01, 0x0f) => Some("PCI native mode controller, supports both channels switched to ISA compatibility mode"),
(0x01, 0x01, 0x80) => Some("ISA Compatibility mode-only controller, supports bus mastering"),
(0x01, 0x01, 0x85) => Some("PCI native mode-only controller, supports bus mastering"),
(0x01, 0x01, 0x8a) => Some("ISA Compatibility mode controller, supports both channels switched to PCI native mode, supports bus mastering"),
(0x01, 0x01, 0x8f) => Some("PCI native mode controller, supports both channels switched to ISA compatibility mode, supports bus mastering"),
(0x01, 0x05, 0x20) => Some("ADMA single stepping"),
(0x01, 0x05, 0x30) => Some("ADMA continuous operation"),
(0x01, 0x06, 0x00) => Some("Vendor specific"),
(0x01, 0x06, 0x01) => Some("AHCI 1.0"),
(0x01, 0x06, 0x02) => Some("Serial Storage Bus"),
(0x01, 0x07, 0x01) => Some("Serial Storage Bus"),
(0x01, 0x08, 0x01) => Some("NVMHCI"),
(0x01, 0x08, 0x02) => Some("NVM Express"),
(0x03, 0x00, 0x00) => Some("VGA controller"),
(0x03, 0x00, 0x01) => Some("8514 controller"),
(0x06, 0x04, 0x00) => Some("Normal decode"),
(0x06, 0x04, 0x01) => Some("Subtractive decode"),
(0x06, 0x08, 0x00) => Some("Transparent mode"),
(0x06, 0x08, 0x01) => Some("Endpoint mode"),
(0x06, 0x09, 0x40) => Some("Primary bus towards host CPU"),
(0x06, 0x09, 0x80) => Some("Secondary bus towards host CPU"),
(0x07, 0x00, 0x00) => Some("8250"),
(0x07, 0x00, 0x01) => Some("16450"),
(0x07, 0x00, 0x02) => Some("16550"),
(0x07, 0x00, 0x03) => Some("16650"),
(0x07, 0x00, 0x04) => Some("16750"),
(0x07, 0x00, 0x05) => Some("16850"),
(0x07, 0x00, 0x06) => Some("16950"),
(0x07, 0x01, 0x00) => Some("SPP"),
(0x07, 0x01, 0x01) => Some("BiDir"),
(0x07, 0x01, 0x02) => Some("ECP"),
(0x07, 0x01, 0x03) => Some("IEEE1284"),
(0x07, 0x01, 0xfe) => Some("IEEE1284 Target"),
(0x07, 0x03, 0x00) => Some("Generic"),
(0x07, 0x03, 0x01) => Some("Hayes/16450"),
(0x07, 0x03, 0x02) => Some("Hayes/16550"),
(0x07, 0x03, 0x03) => Some("Hayes/16650"),
(0x07, 0x03, 0x04) => Some("Hayes/16750"),
(0x08, 0x00, 0x00) => Some("8259"),
(0x08, 0x00, 0x01) => Some("ISA PIC"),
(0x08, 0x00, 0x02) => Some("EISA PIC"),
(0x08, 0x00, 0x10) => Some("IO-APIC"),
(0x08, 0x00, 0x20) => Some("IO(X)-APIC"),
(0x08, 0x01, 0x00) => Some("8237"),
(0x08, 0x01, 0x01) => Some("ISA DMA"),
(0x08, 0x01, 0x02) => Some("EISA DMA"),
(0x08, 0x02, 0x00) => Some("8254"),
(0x08, 0x02, 0x01) => Some("ISA Timer"),
(0x08, 0x02, 0x02) => Some("EISA Timers"),
(0x08, 0x02, 0x03) => Some("HPET"),
(0x08, 0x03, 0x00) => Some("Generic"),
(0x08, 0x03, 0x01) => Some("ISA RTC"),
(0x08, 0x99, 0x01) => Some("TAP Timing Card"),
(0x09, 0x04, 0x00) => Some("Generic"),
(0x09, 0x04, 0x10) => Some("Extended"),
(0x0c, 0x00, 0x00) => Some("Generic"),
(0x0c, 0x00, 0x10) => Some("OHCI"),
(0x0c, 0x03, 0x00) => Some("UHCI"),
(0x0c, 0x03, 0x10) => Some("OHCI"),
(0x0c, 0x03, 0x20) => Some("EHCI"),
(0x0c, 0x03, 0x30) => Some("XHCI"),
(0x0c, 0x03, 0x40) => Some("USB4 Host Interface"),
(0x0c, 0x03, 0x80) => Some("Unspecified"),
(0x0c, 0x03, 0xfe) => Some("USB Device"),
(0x0c, 0x07, 0x00) => Some("SMIC"),
(0x0c, 0x07, 0x01) => Some("KCS"),
(0x0c, 0x07, 0x02) => Some("BT (Block Transfer)"),
(_, _, _) => None
}
}
