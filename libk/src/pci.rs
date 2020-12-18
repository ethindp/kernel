// SPDX-License-Identifier: MPL-2.0
use minivec::MiniVec;
use bit_field::BitField;
use lazy_static::lazy_static;
use log::*;
use spin::RwLock;
use hashbrown::HashMap;

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
pub const SEC_BUS: u32 = 0x19;
pub const SUB_BUS: u32 = 0x1A;
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
    static ref PCI_DEVICES: RwLock<MiniVec<PCIDevice>> = RwLock::new(MiniVec::new());
    static ref DEV_INIT_FUNCS: RwLock<HashMap<(u8, u8, u8), fn(PCIDevice)>> = RwLock::new({
    let mut map: HashMap<(u8, u8, u8), fn(PCIDevice)> = HashMap::new();
    if cfg!(feature="nvme") {
    map.insert((0x01, 0x08, 0x02), crate::nvme::init);
    }
    map
    });
}

/// Contains PCI device properties.
/// This structure contains only static properties that remain unchanged.
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
    pub secondary_bus: u8,
    pub phys_addr: u64,
    pub bars: (u64, u64, u64, u64, u64, u64),
    pub htype: u8,
    pub cis_ptr: u32,
    pub ssid: u16,
    pub ssvid: u16,
    pub exp_rom_base_addr: u32,
    pub caps_ptr: u16,
    pub int_pin: u8,
    pub int_line: u8,
}

// Adds a device to the PCI device list.
fn add_device(device: PCIDevice) {
    let mut devs = PCI_DEVICES.write();
    let l = devs.len();
    devs.reserve(l * 2);
    devs.push(device);
}

pub fn read_dword(phys_addr: usize, addr: u32) -> u32 {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u32> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.read()
}

pub fn read_word(phys_addr: usize, addr: u32) -> u16 {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u16> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.read()
}

pub fn read_byte(phys_addr: usize, addr: u32) -> u8 {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u8> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.read()
}

pub fn write_dword(phys_addr: usize, addr: u32, value: u32) {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u32> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.write(value);
}

pub fn write_word(phys_addr: usize, addr: u32, value: u16) {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u16> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.write(value);
}

pub fn write_byte(phys_addr: usize, addr: u32, value: u8) {
    use voladdress::VolAddress;
    let cfgspace: VolAddress<u8> = unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.write(value);
}

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
                            allocate_phys_range(addr, addr + 4096);
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
                            caps_ptr: read_word(addr as usize, CAP_LIST),
                            int_pin: read_byte(addr as usize, INT_PIN),
                            int_line: read_byte(addr as usize, INT_LINE)
                            };
                            dev.bars = (calculate_bar_addr(&dev, BAR0) as u64, calculate_bar_addr(&dev, BAR1) as u64, calculate_bar_addr(&dev, BAR2) as u64, calculate_bar_addr(&dev, BAR3) as u64, calculate_bar_addr(&dev, BAR4) as u64, calculate_bar_addr(&dev, BAR5) as u64);
                            info!("Detected device of type {} with vendor ID of {:X} and subsystem ID {:X}", classify_program_interface(dev.class, dev.subclass, dev.prog_if).unwrap_or_else(|| classify_subclass(dev.class, dev.subclass).unwrap_or_else(|| classify_class(dev.class).unwrap_or("Unknown Device"))), dev.vendor, dev.ssid);
                            let funcs = DEV_INIT_FUNCS.read();
                            funcs.iter().filter(|(k, _)| k.0 == dev.class && k.1 == dev.subclass && k.2 == dev.prog_if).for_each(|(_, v)| {
                            info!("Found device driver for class={:X}, subclass={:X}, program interface={:X}; initiating initialization sequence", dev.class, dev.subclass, dev.prog_if);
                            (v)(dev.clone());
                            });
                            add_device(dev.clone());
                        }
                    }))));
        let mut devs = PCI_DEVICES.write();
        devs.shrink_to_fit();
    } else {
        error!("No PCI regions");
    }
}

pub async fn init() {
    info!("Initiating PCE bus scan");
    probe().await;
    info!("PCIe scan complete; {} devices found", PCI_DEVICES.read().len());
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

pub async fn find_device(class: u8, subclass: u8, interface: u8) -> Option<PCIDevice> {
    let devs = PCI_DEVICES.read();
    devs.iter().filter(|d| d.class == class && d.subclass == subclass && d.prog_if == interface).cloned().next()
}

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
