extern crate alloc;
use crate::pcidb::*;
use crate::printkln;
use alloc::vec::Vec;
use bit_field::BitField;
use cpuio::*;
use lazy_static::lazy_static;
use spin::Mutex;

const MAX_FUNCTION: u16 = 8;
const MAX_DEVICE: u16 = 32;
const MAX_BUS: u16 = 256;

// These statics are internally tracked by the kernel -- do not modify!
lazy_static! {
// Vector to hold a list of all recognized PCI devices.
    static ref PCI_DEVICES: Mutex<Vec<PCIDevice>> = Mutex::new(Vec::new());
}

/// Contains PCI device properties.
#[derive(Debug, Copy, Clone)]
pub struct PCIDevice {
    /// Public vendor ID of device
    pub vendor: u32,
    /// Device number
    pub device: u32,
    /// Function number
    pub func: u32,
    /// Bus number of this device
    pub bus: u32,
    /// A register used to record status information for PCI bus related events.
    pub status: u32,
    /// Provides control over a device's ability to generate and respond to PCI cycles, where the only functionality guaranteed to be supported by all devices is that when a 0 is written to this register, the device is disconnected from the PCI bus for all accesses except Configuration Space access.
    pub command: u32,
    /// A read-only register that specifies the type of function the device performs.
    pub class: u32,
    /// A read-only register that specifies the specific function the device performs.
    pub subclass: u32,
    /// A read-only register that specifies a register-level programming interface the device has, if it has any at all.
    pub prog_if: u32,
    /// Specifies a revision identifier for a particular device. Valid IDs are allocated by the vendor.
    pub revision_id: u32,
    /// Represents the status of, and allows control of, a devices BIST (built-in self test).
    pub bist: u32,
    /// Identifies the layout of the rest of the header beginning at byte 0x10 of the header and also specifies whether or not the device has multiple functions; a value of 0x00 specifies a general device, a value of 0x01 specifies a PCI-to-PCI bridge, and a value of 0x02 specifies a CardBus bridge. If bit 7 of this register is set, the device has multiple functions; otherwise, it is a single function device.
    pub header_type: u32,
    /// Specifies the latency timer in units of PCI bus clocks.
    pub latency_timer: u32,
    /// Specifies the system cache line size in 32-bit units. A device can limit the number of cache line sizes it can support. If an unsupported value is written to this field, the device will behave as if a value of 0 was written.
    pub cache_line_size: u32,
    /// General PCI device table (GPDT). Only has Some() if header type is 00h.
    pub gen_dev_tbl: Option<GeneralDeviceTable>,
    /// PCI-to-PCI bridge table (P2PBT). Only contains Some() if header type == 01h.
    pub pci_to_pci_bridge_tbl: Option<PCIToPCIBridgeTable>,
    /// PCI-to-CardBus bridge table (P2CBBT). Only contains Some() if header type == 02h.
    pub pci_to_card_bus_bridge_tbl: Option<PCIToCardBusBridgeTable>,
}

/// This table is applicable if the Header Type is 00h.
#[derive(Debug, Copy, Clone)]
pub struct GeneralDeviceTable {
    // Address of all six BARs.
    pub bar0: u32,
    pub bar1: u32,
    pub bar2: u32,
    pub bar3: u32,
    pub bar4: u32,
    pub bar5: u32,
    /// Points to the Card Information Structure and is used by devices that share silicon between CardBus and PCI.
    pub cis_ptr: u32,
    pub subsystem_id: u32,
    pub subsystem_vendor_id: u32,
    pub expansion_rom_addr: u32,
    /// A read-only register that specifies how often the device needs access to the PCI bus (in 1/4 microsecond units).
    pub max_latency: u32,
    /// A read-only register that specifies the burst period length, in 1/4 microsecond units, that the device needs (assuming a 33 MHz clock rate).
    pub min_grant: u32,
    /// Specifies which interrupt pin the device uses, where a value of 0x01 is INTA#, 0x02 is INTB#, 0x03 is INTC#, 0x04 is INTD#, and 0x00 means the device does not use an interrupt pin.
    pub interrupt_pin: u32,
    /// Specifies which input of the system interrupt controllers the device's interrupt pin is connected to and is implemented by any device that makes use of an interrupt pin. For the x86 architecture this register corresponds to the PIC IRQ numbers 0-15 (and not I/O APIC IRQ numbers) and a value of 0xFF defines no connection.
    pub interrupt_line: u32,
    /// Points (i.e. an offset into this function's configuration space) to a linked list of new capabilities implemented by the device. Used if bit 4 of the status register (Capabilities List bit) is set to 1. The bottom two bits are reserved and should be masked before the Pointer is used to access the Configuration Space.
    pub caps_ptr: u32,
}

/// This table is applicable if the Header Type is 01h (PCI-to-PCI bridge)
#[derive(Debug, Copy, Clone)]
pub struct PCIToPCIBridgeTable {
    pub bar0: u32,
    pub bar1: u32,
    pub sec_latency_timer: u32,
    pub sub_bus: u32,
    pub sec_bus: u32,
    pub prim_bus: u32,
    pub sec_status: u32,
    pub io_limit: u32,
    pub io_base: u32,
    pub mem_limit: u32,
    pub mem_base: u32,
    pub prefetch_mem_limit: u32,
    pub prefetch_mem_base: u32,
    pub prefetch_base_upper32: u32,
    pub prefetch_limit_upper32: u32,
    pub io_limit_upper16: u32,
    pub io_base_upper16: u32,
    pub caps_ptr: u32,
    pub expansion_rom_addr: u32,
    pub bridge_control: u32,
    pub interrupt_pin: u32,
    pub interrupt_line: u32,
}

/// This table is applicable if the Header Type is 02h (PCI-to-CardBus bridge)
#[derive(Debug, Copy, Clone)]
pub struct PCIToCardBusBridgeTable {
    pub exca_base_addr: u32,
    pub sec_status: u32,
    pub caps_lst_offset: u32,
    pub card_bus_latency_timer: u32,
    pub sub_bus: u32,
    pub card_bus_bus: u32,
    pub pci_bus: u32,
    pub mem_base_addr0: u32,
    pub mem_limit0: u32,
    pub mem_base_addr1: u32,
    pub mem_limit1: u32,
    pub io_base_addr0: u32,
    pub io_base_limit0: u32,
    pub io_base_addr1: u32,
    pub io_base_limit1: u32,
    pub bridge_control: u32,
    pub interrupt_pin: u32,
    pub interrupt_line: u32,
    pub subsystem_vendor_id: u32,
    pub subsystem_device_id: u32,
    pub legacy_base_addr: u32,
}

// Adds a device to the PCI device list.
fn add_device(device: PCIDevice) {
    PCI_DEVICES.lock().push(device);
}

/// Reads a word from a PCI bus, device and function using the given offset and returns it.
pub fn read_word(bus: u16, slot: u16, func: u16, offset: u16) -> u32 {
    let lbus = bus as u32;
    let lslot = slot as u32;
    let lfunc = func as u32;
    unsafe {
        outl(
            (((lbus << 16) as u32)
                | ((lslot << 11) as u32)
                | ((lfunc << 8) as u32)
                | ((offset as u32) & 0xfc)
                | (0x80000000)) as u32,
            0xCF8,
        );
        inl(0xCFC) >> ((offset & 2) * 8) & 0xFFFF
    }
}

// Here there be internals.

fn get_vendor_id(bus: u16, device: u16, function: u16) -> u32 {
    read_word(bus, device, function, 0)
}

fn get_device_id(bus: u16, device: u16, function: u16) -> u32 {
    read_word(bus, device, function, 2)
}

fn get_class_id(bus: u16, device: u16, function: u16) -> u32 {
    (read_word(bus, device, function, 0xA) & !0x00FF) >> 8
}

fn get_prog_if(bus: u16, device: u16, function: u16) -> u32 {
    read_word(bus, device, function, 0x8).get_bits(8..15)
}

fn get_header_type(bus: u16, device: u16, function: u16) -> u32 {
    read_word(bus, device, function, 0x0C).get_bits(16..23)
}

fn get_subclass_id(bus: u16, device: u16, function: u16) -> u32 {
    (read_word(bus, device, function, 0x08) & !0xFF00)
}

fn get_status(bus: u16, device: u16, function: u16) -> u32 {
    read_word(bus, device, function, 0x04).get_bits(24..31)
}

fn get_command(bus: u16, device: u16, function: u16) -> u32 {
    read_word(bus, device, function, 0x04).get_bits(16..23)
}

fn get_rev(bus: u16, device: u16, function: u16) -> u32 {
    read_word(bus, device, function, 0x08).get_bits(0..7)
}

pub fn probe() {
    for bus in 0..MAX_BUS {
        for slot in 0..MAX_DEVICE {
            for function in 0..MAX_FUNCTION {
                // Get vendor, device, class and subclass codes.
                let vendor = get_vendor_id(bus, slot, function);
                if vendor == 0xFFFF {
                    continue;
                }
                let device = get_device_id(bus, slot, function);
                let class = get_class_id(bus, slot, function);
                let subclass = get_subclass_id(bus, slot, function);
                printkln!(
                    "PCI: probe: found {} {} ({})",
                    get_vendor_string(vendor),
                    get_device_string(device),
                    get_subclass_string(class, subclass)
                );
                printkln!("PCI: probe: codes: vendor = {:X}h, device = {:X}h, class = {:X}h, subclass = {:X}h, prog if={:X}h, rev={:X}h, status={:X}h, command={:X}h, bus = {:X}h, slot = {:X}h, function = {:X}h", vendor, device, class, subclass, get_prog_if(bus, slot, function), get_rev(bus, slot, function), get_status(bus, slot, function), get_command(bus, slot, function), bus, slot, function);
                // This part is the longest part of this function thus far. Here we construct the PCI device structure and its linked structures, if applicable.
                // Construction happens in this order:
                // 1. Initialize static (easily calculable/readable) data.
                // 2. Use "conditional initialization" to initialize dynamic data that requires extra reads.
                // Conditional initialization is the term I use when I take advantage of conditional statements being expressions and "conditionally" initialize parts of (or entire) data structures with them, as I do here.
                let pcidev = PCIDevice {
                    // Non-conditional initialization.
                    vendor: vendor,
                    device: device,
                    func: function as u32,
                    bus: bus as u32,
                    status: get_status(bus, slot, function),
                    command: get_command(bus, slot, function),
                    class: class,
                    subclass: subclass,
                    prog_if: get_prog_if(bus, slot, function),
                    revision_id: get_rev(bus, slot, function),
                    bist: read_word(bus, slot, function, 0xF).get_bits(24..31),
                    header_type: get_header_type(bus, slot, function),
                    latency_timer: read_word(bus, slot, function, 0x0C).get_bits(8..15),
                    cache_line_size: read_word(bus, slot, function, 0x0C).get_bits(0..7),
                    // Determine header type and set up appropriate structures from there
                    // Conditional initialization.
                    gen_dev_tbl: if read_word(bus, slot, function, 0x0C).get_bits(16..23) == 0x00 {
                        Some(GeneralDeviceTable {
                            bar0: (read_word(bus, slot, function, 0x10) >> 24),
                            bar1: (read_word(bus, slot, function, 0x14) >> 24),
                            bar2: (read_word(bus, slot, function, 0x18) >> 24),
                            bar3: (read_word(bus, slot, function, 0x1C) >> 24),
                            bar4: (read_word(bus, slot, function, 0x20) >> 24),
                            bar5: (read_word(bus, slot, function, 0x24) >> 24),
                            cis_ptr: read_word(bus, slot, function, 0x28).get_bits(24..31),
                            subsystem_id: read_word(bus, slot, function, 0x2C).get_bits(24..31),
                            subsystem_vendor_id: read_word(bus, slot, function, 0x2C)
                                .get_bits(16..23),
                            expansion_rom_addr: read_word(bus, slot, function, 0x30)
                                .get_bits(24..31),
                            caps_ptr: read_word(bus, slot, function, 0x34).get_bits(16..23),
                            max_latency: read_word(bus, slot, function, 0x3C).get_bits(24..31),
                            min_grant: read_word(bus, slot, function, 0x3C).get_bits(16..23),
                            interrupt_pin: read_word(bus, slot, function, 0x3C).get_bits(8..15),
                            interrupt_line: read_word(bus, slot, function, 0x3C).get_bits(0..7),
                        })
                    } else {
                        None
                    },
                    pci_to_pci_bridge_tbl: if read_word(bus, slot, function, 0xF).get_bits(16..23)
                        == 0x01
                    {
                        Some(PCIToPCIBridgeTable {
                            bar0: read_word(bus, slot, function, 0x10).get_bits(24..31),
                            bar1: read_word(bus, slot, function, 0x14).get_bits(24..31),
                            sec_latency_timer: read_word(bus, slot, function, 0x18)
                                .get_bits(24..31),
                            sub_bus: read_word(bus, slot, function, 0x18).get_bits(16..23),
                            sec_bus: read_word(bus, slot, function, 0x18).get_bits(8..15),
                            prim_bus: read_word(bus, slot, function, 0x18).get_bits(0..7),
                            sec_status: read_word(bus, slot, function, 0x1C).get_bits(24..31),
                            io_limit: read_word(bus, slot, function, 0x1C).get_bits(16..23),
                            io_base: read_word(bus, slot, function, 0x1C).get_bits(8..15),
                            mem_limit: read_word(bus, slot, function, 0x20).get_bits(24..31),
                            mem_base: read_word(bus, slot, function, 0x20).get_bits(16..23),
                            prefetch_mem_limit: read_word(bus, slot, function, 0x24)
                                .get_bits(24..31),
                            prefetch_mem_base: read_word(bus, slot, function, 0x24)
                                .get_bits(16..23),
                            prefetch_base_upper32: read_word(bus, slot, function, 0x28)
                                .get_bits(24..31),
                            prefetch_limit_upper32: read_word(bus, slot, function, 0x2C)
                                .get_bits(24..31),
                            io_limit_upper16: read_word(bus, slot, function, 0x30).get_bits(24..31),
                            io_base_upper16: read_word(bus, slot, function, 0x30).get_bits(16..23),
                            caps_ptr: read_word(bus, slot, function, 0x34).get_bits(16..23),
                            expansion_rom_addr: read_word(bus, slot, function, 0x38)
                                .get_bits(24..31),
                            bridge_control: read_word(bus, slot, function, 0x3C).get_bits(24..31),
                            interrupt_pin: read_word(bus, slot, function, 0x3C).get_bits(16..23),
                            interrupt_line: read_word(bus, slot, function, 0x3C).get_bits(8..15),
                        })
                    } else {
                        None
                    },
                    pci_to_card_bus_bridge_tbl: if read_word(bus, slot, function, 0xF)
                        .get_bits(16..23)
                        == 0x02
                    {
                        Some(PCIToCardBusBridgeTable {
                            exca_base_addr: read_word(bus, slot, function, 0x10).get_bits(24..31),
                            sec_status: read_word(bus, slot, function, 0x14).get_bits(24..31),
                            caps_lst_offset: read_word(bus, slot, function, 0x14).get_bits(8..15),
                            card_bus_latency_timer: read_word(bus, slot, function, 0x18)
                                .get_bits(24..31),
                            sub_bus: read_word(bus, slot, function, 0x18).get_bits(16..23),
                            card_bus_bus: read_word(bus, slot, function, 0x18).get_bits(8..15),
                            pci_bus: read_word(bus, slot, function, 0x18).get_bits(0..7),
                            mem_base_addr0: read_word(bus, slot, function, 0x1C).get_bits(24..31),
                            mem_limit0: read_word(bus, slot, function, 0x20).get_bits(24..31),
                            mem_base_addr1: read_word(bus, slot, function, 0x24).get_bits(24..31),
                            mem_limit1: read_word(bus, slot, function, 0x28).get_bits(24..31),
                            io_base_addr0: read_word(bus, slot, function, 0x2C).get_bits(24..31),
                            io_base_limit0: read_word(bus, slot, function, 0x30).get_bits(24..31),
                            io_base_addr1: read_word(bus, slot, function, 0x34).get_bits(24..31),
                            io_base_limit1: read_word(bus, slot, function, 0x38).get_bits(24..31),
                            bridge_control: read_word(bus, slot, function, 0x3C).get_bits(24..31),
                            interrupt_pin: read_word(bus, slot, function, 0x3C).get_bits(16..23),
                            interrupt_line: read_word(bus, slot, function, 0x3C).get_bits(8..15),
                            subsystem_vendor_id: read_word(bus, slot, function, 0x40)
                                .get_bits(24..31),
                            subsystem_device_id: read_word(bus, slot, function, 0x40)
                                .get_bits(16..23),
                            legacy_base_addr: read_word(bus, slot, function, 0x44).get_bits(24..31),
                        })
                    } else {
                        None
                    },
                };
                add_device(pcidev);
            }
        }
    }
}

pub fn init() {
    probe();
}

pub fn get_devices() -> Vec<PCIDevice> {
    let mut devices: Vec<PCIDevice> = Vec::new();
    for device in PCI_DEVICES.lock().iter() {
        devices.push(*device);
    }
    devices
}
