// SPDX-License-Identifier: MPL-2.0
use alloc::boxed::Box;
use alloc::vec::Vec;
use async_recursion::async_recursion;
use bit_field::BitField;
use heapless::LinearMap;
use log::*;
use spin::{mutex::ticket::TicketMutex, Lazy};
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

static PCI_DEVICES: Lazy<TicketMutex<Vec<PciDevice>>> = Lazy::new(|| TicketMutex::new(Vec::new()));

/// Contains PCI device properties.
/// This structure contains only static properties that remain unchanged.
#[repr(C)]
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct PciDevice {
    /// Segment group of the device.
    pub segment_group: u16,
    /// Bus number of the device.
    pub bus: u8,
    /// Slot (device) number of the device.
    pub device: u8,
    /// Function number of the device. A multifunction device is free to implement
    /// multiple functions, so a different function does not always mean that this is a
    /// different device. However, the system treats each (sg, bus, slot, function)
    /// combination as a new device.
    pub function: u8,
    /// Physical memory address in PCIe configuration space for this device.
    pub phys_addr: u64,
    /// This field identifies the manufacturer of the device. Valid vendor
    /// identifiers are allocated by the PCI SIG to ensure uniqueness.
    pub vendor_id: u16,
    /// This field identifies the particular device. This identifier is allocated by
    /// the vendor.
    pub device_id: u16,
    /// This register specifies a device specific revision identifier.
    pub revision_id: u8,
    /// This byte identifies the layout of the second part of the predefined header
    /// beginning at byte 0x10.
    pub header_type: u8,
    /// Used to identify a multi-function device. If false, then the device is
    /// single-function. If true, then the device has multiple functions.
    pub multifunction: bool,
    /// The Class Code register is read-only and is used to identify the generic
    /// function of the device and, in some cases, a specific register-level programming
    /// interface.
    pub class: (u8, u8, u8),
    /// Base address registers (BARs)
    pub bars: LinearMap<u8, usize, 6>,
    /// Contains a unique device ID. Device drivers may use this (e.g.: to allow interrupt->driver
    /// communication).
    pub unique_dev_id: u64,
}

// Adds a device to the PCI device list.
#[inline]
fn add_device(device: PciDevice) {
    PCI_DEVICES.lock().push(device);
}

fn read_dword(phys_addr: usize, addr: u32) -> u32 {
    let cfgspace: VolAddress<u32, Safe, ()> =
        unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.read()
}

fn write_dword(phys_addr: usize, addr: u32, value: u32) {
    let cfgspace: VolAddress<u32, (), Safe> =
        unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.write(value);
}

/// Probes the PCI bus.
#[cold]
pub async fn probe() {
    if let Err(e) = crate::acpi::get_pci_regions() {
        error!("PCIe is not supported; terminating scan");
        error!("Additional error information: {:?}", e);
        return;
    }
    let regions = crate::acpi::get_pci_regions().unwrap();
    for sg in (0..MAX_SG).filter(|sg| regions.physical_address(*sg as u16, 0, 0, 0).is_some()) {
        check_sg(sg as _).await;
    }
}

#[async_recursion]
async fn check_sg(sg: u16) {
    let regions = crate::acpi::get_pci_regions().unwrap();
    let addr = regions.physical_address(sg, 0, 0, 0).unwrap() as usize;
    let header_type = read_dword(addr, 0x0C).get_bits(16..24);
    if (header_type & 0x80) == 0 {
        check_bus(sg, 0).await;
    } else {
        for function in 0..MAX_FUNCTION {
            match regions.physical_address(sg, 0, 0, function as _) {
                Some(addr) => {
                    if read_dword(addr as usize, 0x00).get_bits(0..16) != 0xFFFF {
                        break;
                    } else {
                        check_bus(sg, function as _).await;
                    }
                }
                None => break,
            }
        }
    }
}

#[async_recursion]
async fn check_bus(sg: u16, bus: u8) {
    let regions = crate::acpi::get_pci_regions().unwrap();
    for device in (0..MAX_DEVICE)
        .filter(|device| regions.physical_address(sg, bus, *device as _, 0).is_some())
    {
        check_device(sg, bus, device as _).await;
    }
}

#[async_recursion]
async fn check_device(sg: u16, bus: u8, device: u8) {
    let regions = crate::acpi::get_pci_regions().unwrap();
    for function in (0..MAX_FUNCTION).filter(|function| {
        regions
            .physical_address(sg, bus, device, *function as _)
            .is_some()
    }) {
        check_function(sg, bus, device, function as _).await;
    }
}

#[async_recursion]
async fn check_function(sg: u16, bus: u8, device: u8, function: u8) {
    let addr = crate::acpi::get_pci_regions()
        .unwrap()
        .physical_address(sg, bus, device, function)
        .unwrap() as usize;
    let data = read_dword(addr, 0x08);
    if data.get_bits(24..32) == 0x06 && data.get_bits(16..24) == 0x04 {
        info!(
            "SBDF {:X} is secondary bus, enumerating",
            *0u32
                .set_bits(0..16, sg as u32)
                .set_bits(16..24, bus as u32)
                .set_bits(24..32, device as u32)
                .set_bits(32..40, function as u32)
        );
        let data = read_dword(addr, 18);
        check_bus(sg, data.get_bits(8..15) as u8).await;
    }
    let mut dev: PciDevice = Default::default();
    unsafe {
        random::rdrand64(&mut dev.unique_dev_id);
    }
    dev.segment_group = sg;
    dev.bus = bus;
    dev.function = function;
    dev.device = device;
    dev.phys_addr = addr as _;
    let data = read_dword(addr, 0x00);
    dev.vendor_id = data.get_bits(0..16) as u16;
    dev.device_id = data.get_bits(16..32) as u16;
    let data = read_dword(addr, 0x08);
    dev.class = (
        data.get_bits(24..32) as _,
        data.get_bits(16..24) as _,
        data.get_bits(8..16) as _,
    );
    dev.revision_id = data.get_bits(0..8) as _;
    let data = read_dword(addr, 0x0C);
    dev.header_type = data.get_bits(16..24).get_bits(0..7) as _;
    dev.multifunction = data.get_bits(16..24).get_bit(7);
    dev.bars = LinearMap::new();
    if dev.header_type == 0x00 {
        dev.bars.insert(0, calculate_bar_addr(&dev, BAR0)).unwrap();
        dev.bars.insert(1, calculate_bar_addr(&dev, BAR1)).unwrap();
        dev.bars.insert(2, calculate_bar_addr(&dev, BAR2)).unwrap();
        dev.bars.insert(3, calculate_bar_addr(&dev, BAR3)).unwrap();
        dev.bars.insert(4, calculate_bar_addr(&dev, BAR4)).unwrap();
        dev.bars.insert(5, calculate_bar_addr(&dev, BAR5)).unwrap();
    } else if dev.header_type == 0x01 {
        dev.bars.insert(0, calculate_bar_addr(&dev, BAR0)).unwrap();
        dev.bars.insert(1, calculate_bar_addr(&dev, BAR1)).unwrap();
    } else if dev.header_type == 0x02 {
        let cbbar = read_dword(addr, BAR0) as usize;
        let membar0 = *read_dword(addr, 0x1C).set_bits(32..64, read_dword(addr, 0x20)) as usize;
        let membar1 = *read_dword(addr, 0x24).set_bits(32..64, read_dword(addr, 0x28)) as usize;
        let iobar0 = *read_dword(addr, 0x2C).set_bits(32..64, read_dword(addr, 0x30)) as usize;
        let iobar1 = *read_dword(addr, 0x34).set_bits(32..64, read_dword(addr, 0x38)) as usize;
        dev.bars.insert(0, cbbar).unwrap();
        dev.bars.insert(1, membar0).unwrap();
        dev.bars.insert(2, membar1).unwrap();
        dev.bars.insert(3, iobar0).unwrap();
        dev.bars.insert(4, iobar1).unwrap();
    }
    info!(
        "{:X}:{:X}:{:X}:{:X}: Found {} ({}) with vendor ID {:X} and device ID {:X}",
        dev.segment_group,
        dev.bus,
        dev.device,
        dev.function,
        classify_class(dev.class.0).unwrap(),
        classify_subclass(dev.class.0, dev.class.1).unwrap(),
        dev.vendor_id,
        dev.device_id
    );
    add_device(dev);
}

/// Initializes the PCI subsystem.
#[cold]
pub async fn init() {
    info!("Initiating PCE bus scan");
    probe().await;
    info!(
        "PCIe scan complete; {} devices found",
        PCI_DEVICES.lock().len()
    );
}

#[inline]
fn calculate_bar_addr(dev: &PciDevice, addr: u32) -> usize {
    let bar1 = read_dword(dev.phys_addr as _, addr);
    if !bar1.get_bit(0) {
        match bar1.get_bits(1..=2) {
            0 => (bar1 & 0xFFFF_FFF0) as usize,
            1 => (bar1 & 0xFFF0) as usize,
            2 => {
                let bar2 = read_dword(
                    dev.phys_addr as _,
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
pub async fn find_device(class: u8, subclass: u8, interface: u8) -> Option<PciDevice> {
    PCI_DEVICES
        .lock()
        .iter()
        .filter(|d| d.class.0 == class && d.class.1 == subclass && d.class.2 == interface)
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
