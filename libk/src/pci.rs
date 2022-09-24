// SPDX-License-Identifier: MPL-2.0
use crate::memory::{allocate_phys_range, free_range};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use async_recursion::async_recursion;
use bit_field::BitField;
use heapless::LinearMap;
use log::*;
use spin::{mutex::ticket::TicketMutex, Lazy};
use voladdress::*;
use x86_64::instructions::random::RdRand;
use x86_64::structures::paging::page_table::PageTableFlags;

include!(concat!(env!("OUT_DIR"), "/pciids.rs"));

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
const SUBSYS_VENDOR_ID: u32 = 0x2C;
const SUBSYS_ID: u32 = 0x2E;
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
const MEM_LIMIT: u32 = 0x22;
const PREF_MEM_BASE: u32 = 0x24;
const PREF_MEM_LIMIT: u32 = 0x26;
const PREF_MEM_BASE_UPPER32: u32 = 0x28;
const PREF_MEM_LIMIT_UPPER32: u32 = 0x2C;
const IO_BASE_UPPER16: u32 = 0x30;
const IO_LIMIT_UPPER32: u32 = 0x32;
const ROM_ADDR1: u32 = 0x38;
const BRIDGE_CTL: u32 = 0x3E;
const CB_CAP_LIST: u32 = 0x14;
const CB_SEC_STATUS: u32 = 0x16;
const CB_PRIM_BUS: u32 = 0x18;
const CB_CARD_BUS: u32 = 0x19;
const CB_SUB_BUS: u32 = 0x1A;
const CB_LAT_TMR: u32 = 0x1B;
const CB_MEMBASE0: u32 = 0x1C;
const CB_MEMLIMIT0: u32 = 0x20;
const CB_MEMBASE1: u32 = 0x24;
const CB_MEMLIMIT1: u32 = 0x28;
const CB_IO_BASE0: u32 = 0x2C;
const CB_IO_BASE0_HI: u32 = 0x2E;
const CB_IO_LIMIT0: u32 = 0x30;
const CB_IO_LIMIT0_HI: u32 = 0x32;
const CB_IO_BASE1: u32 = 0x34;
const CB_IO_BASE1_HI: u32 = 0x36;
const CB_IO_LIMIT1: u32 = 0x38;
const CB_IO_LIMIT1_HI: u32 = 0x3A;
const CB_BR_CTL: u32 = 0x3E;
const CB_SUBSYS_VENDOR_ID: u32 = 0x40;
const CB_SUBSYS_ID: u32 = 0x42;
const CB_LEG_MODE_BASE: u32 = 0x44;

static PCI_DEVICES: Lazy<TicketMutex<Vec<PciDevice>>> = Lazy::new(|| TicketMutex::new(Vec::new()));

/// Contains PCI device properties.
/// This structure contains only static properties that remain unchanged.
#[repr(C)]
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct PciDevice {
    /// Segment group (domain) of the device.
    pub domain: u16,
    /// Bus number of the device.
    pub bus: u8,
    /// Slot (device) number of the device.
    pub device: u8,
    /// Function number of the device. A multifunction device is free to implement
    /// multiple functions, so a different function does not always mean that this is a
    /// different device. However, the system treats each (domain, bus, slot, function)
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
    /// Base address registers (BARs). Also holds limits for BAR ranges in tuple element 1.
    pub bars: LinearMap<u8, (u64, u64), 6>,
    /// List of capabilities supported by this device. Also contains addresses to each capability.
    pub caps: Capabilities,
    /// Contains a unique device ID. Device drivers may use this (e.g.: to allow interrupt->driver
    /// communication).
    pub unique_dev_id: u64,
}

/// Contains capability lists for a device
#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Capabilities {
    /// PCI-compatible capabilities
    pub pci: BTreeMap<u8, u64>,
    /// Extended capabilities
    pub extended: BTreeMap<u16, u64>,
}

// Adds a device to the PCI device list.
#[inline]
fn add_device(device: PciDevice) {
    PCI_DEVICES.lock().push(device);
}

#[track_caller]
#[inline]
fn read_dword(phys_addr: usize, addr: u32) -> u32 {
    let cfgspace: VolAddress<u32, Safe, ()> =
        unsafe { VolAddress::new(phys_addr + (addr as usize)) };
    cfgspace.read()
}

#[track_caller]
#[inline]
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
        debug!("Scanning domain {:X}", sg);
        check_sg(sg as _).await;
    }
}

#[async_recursion]
#[inline]
async fn check_sg(sg: u16) {
    let regions = crate::acpi::get_pci_regions().unwrap();
    let addr = regions.physical_address(sg, 0, 0, 0).unwrap() as usize;
    allocate_phys_range(addr as u64, (addr as u64) + 0x1000, true, None);
    let header_type = read_dword(addr, 0x0C).get_bits(16..24);
    if !header_type.get_bit(7) {
        debug!("Scanning bus 0");
        check_bus(sg, 0).await;
    } else {
        for function in 0..MAX_FUNCTION {
            debug!("Scanning function {:X}", function);
            match regions.physical_address(sg, 0, 0, function as _) {
                Some(_) => check_bus(sg, function as _).await,
                None => break,
            }
        }
    }
}

#[async_recursion]
#[inline]
async fn check_bus(sg: u16, bus: u8) {
    let regions = crate::acpi::get_pci_regions().unwrap();
    for device in (0..MAX_DEVICE)
        .filter(|device| regions.physical_address(sg, bus, *device as _, 0).is_some())
    {
        debug!("Scanning device {:X}", device);
        check_device(sg, bus, device as _).await;
    }
}

#[async_recursion]
#[inline]
async fn check_device(sg: u16, bus: u8, device: u8) {
    let regions = crate::acpi::get_pci_regions().unwrap();
    for function in (0..MAX_FUNCTION).filter(|function| {
        regions
            .physical_address(sg, bus, device, *function as _)
            .is_some()
    }) {
        debug!("Scanning function {:X}", function);
        check_function(sg, bus, device, function as _).await;
    }
}

#[async_recursion]
#[inline]
async fn check_function(sg: u16, bus: u8, device: u8, function: u8) {
    let addr = crate::acpi::get_pci_regions()
        .unwrap()
        .physical_address(sg, bus, device, function)
        .unwrap() as usize;
    allocate_phys_range(addr as u64, (addr as u64) + 0x1000, true, None);
    let vid_did = read_dword(addr, 0x00);
    if vid_did.get_bits(0..16) == 0xFFFF && vid_did.get_bits(16..32) == 0xFFFF {
        free_range(addr as u64, (addr as u64) + 0x1000);
        return;
    }
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
        debug!("Scanning bus {:X}", data.get_bits(8..15));
        check_bus(sg, data.get_bits(8..15) as u8).await;
    }
    let mut dev: PciDevice = Default::default();
    dev.unique_dev_id = RdRand::new().unwrap().get_u64().unwrap();
    dev.domain = sg;
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
    info!(
        "{:X}:{:X}:{:X}:{:X}: Found {} ({}) with vendor ID {:X} and device ID {:X}",
        dev.domain,
        dev.bus,
        dev.device,
        dev.function,
        classify_class(dev.class.0).unwrap_or("unknown"),
        classify_device(dev.class.0, dev.class.1, dev.class.2).unwrap_or("unknown"),
        dev.vendor_id,
        dev.device_id
    );
    let data = read_dword(addr, 0x0C);
    dev.header_type = data.get_bits(16..24).get_bits(0..7) as _;
    dev.multifunction = data.get_bits(16..24).get_bit(7);
    dev.bars = LinearMap::new();
    let mut idx = 0;
    let mut inc = 0;
    loop {
        idx += inc;
        if (dev.header_type == 0x00 && idx > 5)
            || (dev.header_type == 0x01 && idx > 1)
            || (dev.header_type == 0x02 && idx > 0)
        {
            break;
        }
        let real_idx = match idx {
            0 => BAR0,
            1 => BAR1,
            2 => BAR2,
            3 => BAR3,
            4 => BAR4,
            5 => BAR5,
            _ => 0,
        };
        let oldbar = read_dword(addr, real_idx);
        if oldbar == 0x00 {
            inc = 1;
            continue;
        }
        let oldbar2 = if !oldbar.get_bit(0) && oldbar.get_bits(1..=2) == 0x02 {
            read_dword(addr, real_idx + 4)
        } else {
            0
        };
        write_dword(addr, real_idx, u32::MAX);
        if !oldbar.get_bit(0) && oldbar.get_bits(1..=2) == 0x02 {
            write_dword(addr, real_idx + 4, u32::MAX);
        }
        let mut bar = read_dword(addr, real_idx);
        let bar2 = if !oldbar.get_bit(0) && oldbar.get_bits(1..=2) == 0x02 {
            read_dword(addr, real_idx + 4)
        } else {
            0
        };
        write_dword(addr, real_idx, oldbar);
        if bar2 != 0x00 {
            write_dword(addr, real_idx + 4, oldbar2);
        }
        if bar2 == 0x00 {
            let mut bar = if !oldbar.get_bit(0) {
                *bar.set_bits(0..4, 0)
            } else {
                *bar.set_bits(0..2, 0)
            };
            bar = (!bar) + 1;
            dev.bars.insert(idx, (oldbar as u64, bar as u64)).unwrap();
            debug!("Barcheck: {:X}, {:X}", bar, bar as u64);
            allocate_phys_range(
                oldbar as u64,
                (oldbar as u64) + (bar as u64),
                true,
                Some(
                    PageTableFlags::WRITE_THROUGH
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::NO_CACHE,
                ),
            );
        } else {
            let mut bar = (bar2 as u64) << 32 | (bar as u64);
            let oldbar = (oldbar2 as u64) << 32 | (oldbar as u64);
            bar.set_bits(0..4, 0);
            bar = (!bar) + 1;
            dev.bars.insert(idx, (oldbar, bar)).unwrap();
            allocate_phys_range(
                oldbar,
                oldbar + bar,
                true,
                Some(
                    PageTableFlags::WRITE_THROUGH
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::NO_CACHE,
                ),
            );
        }
        if oldbar.get_bits(1..=2) == 0x02 {
            inc = 2;
        } else {
            inc = 1;
        }
    }
    // Iterate through permissions
    if read_dword(addr, STATUS).get_bit(4) {
        // We have a caps list
        let mut cap_addr = addr + (read_dword(addr, CAP_LIST) as usize);
        loop {
            let data = read_dword(cap_addr, 0x00);
            let cap_id = data.get_bits(0..8);
            let next_ptr = data.get_bits(8..16);
            if next_ptr == 0x00 {
                break;
            }
            info!(
                "Found {} ({}) capability at addr {:X}",
                match cap_id {
                    0x00 => "null",
                    0x01 => "power management",
                    0x02 => "AGP",
                    0x03 => "VPD",
                    0x04 => "slot identification",
                    0x05 => "MSI",
                    0x06 => "compact PCI hot swap",
                    0x07 => "PCI-X",
                    0x08 => "hyper transport",
                    0x09 => "vendor specific",
                    0x0A => "debug port",
                    0x0B => "compact PCI central resource control",
                    0x0C => "PCI hot plug",
                    0x0D => "PCI bridge subsystem vendor ID",
                    0x0E => "AGP 8x",
                    0x0F => "secure device",
                    0x10 => "PCI express",
                    0x11 => "MSI-X",
                    0x12 => "SATA data/index configuration",
                    0x13 => "advanced features",
                    0x14 => "enhanced allocation",
                    0x15 => "flattening portal bridge",
                    _ => "reserved",
                },
                cap_id,
                cap_addr
            );
            let _ = dev.caps.pci.entry(cap_id as u8).or_insert(cap_addr as u64);
            cap_addr += next_ptr as usize;
        }
        // Loop through extended capabilities
        cap_addr = addr + 0x100;
        loop {
            let data = read_dword(cap_addr, 0x00);
            let cap_id = data.get_bits(0..16);
            let cap_ver = data.get_bits(16..20);
            let next_ptr = data.get_bits(20..32);
            if data == 0x00000000 || next_ptr == 0x0000 {
                break;
            }
            info!(
                "Found {} ({}) extended capability at addr {:X}, ver. {}",
                match cap_id {
                    0x0000 => "null",
                    0x0001 => "advanced error reporting",
                    0x0002 | 0x0009 => "virtual channel",
                    0x0003 => "device serial number",
                    0x0004 => "power budgeting",
                    0x0005 => "root complex link declaration",
                    0x0006 => "root complex internal link control",
                    0x0007 => "root complex event collector endpoint association",
                    0x0008 => "multi-function virtual channel",
                    0x000A => "root complex register block",
                    0x000B => "vendor specific",
                    0x000C => "configuration access correlation",
                    0x000D => "access control services",
                    0x000E => "alternative routing-ID interpretation",
                    0x000F => "address translation services",
                    0x0010 => "single root IO virtualization",
                    0x0011 => "multi-root IO virtualization",
                    0x0012 => "multicast",
                    0x0013 => "page request interface",
                    0x0014 => "AMD reserved",
                    0x0015 => "resizable BAR",
                    0x0016 => "dynamic power allocation",
                    0x0017 => "TPH requester",
                    0x0018 => "latency tolerance reporting",
                    0x0019 => "secondary PCI express",
                    0x001A => "protocol multiplexing",
                    0x001B => "process address space ID",
                    0x001C => "LN requester",
                    0x001D => "downstream port containment",
                    0x001E => "L1 PM substates",
                    0x001F => "precision time measurement",
                    0x0020 => "M-PCIe",
                    0x0021 => "FRS queueing",
                    0x0022 => "Readyness time reporting",
                    0x0023 => "designated vendor specific",
                    0x0024 => "VF resizable BAR",
                    0x0025 => "data link feature",
                    0x0026 => "physical layer 16.0 GT/s",
                    0x0027 => "receiver lane margining",
                    0x0028 => "hierarchy ID",
                    0x0029 => "native PCIe enclosure management",
                    0x002A => "physical layer 32.0 GT/s",
                    0x002B => "alternate protocol",
                    0x002C => "system firmware intermediary",
                    _ => "reserved",
                },
                cap_id,
                cap_addr,
                cap_ver
            );
            let _ = dev
                .caps
                .extended
                .entry(cap_id as u16)
                .or_insert(cap_addr as u64);
            cap_addr += next_ptr as usize;
        }
    }
    add_device(dev);
}

/// Initializes the PCI subsystem.
#[cold]
pub async fn init() {
    info!("Initiating PCIe bus scan");
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
                let bar2 = read_dword(dev.phys_addr as _, addr + 4);
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
const fn classify_device(class: u8, subclass: u8, interface: u8) -> Option<&'static str> {
    match classify_prog_if(class, subclass, interface) {
        Some(r) => Some(r),
        None => match classify_subclass(class, subclass) {
            Some(r) => Some(r),
            None => classify_class(class),
        },
    }
}
