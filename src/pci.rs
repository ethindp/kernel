use crate::interrupts::get_tick_count;
use crate::printkln;
use bit_field::BitField;
use cpuio::*;
use lazy_static::lazy_static;
use spin::Mutex;

const MAX_FUNCTION: usize = 8;
const MAX_DEVICE: usize = 32;
const MAX_BUS: usize = 256;

// These statics are internally tracked by the kernel -- do not modify!
lazy_static! {
// Array to hold a list of all recognized PCI devices.
    static ref PCI_DEVICES: Mutex<[Option<PCIDevice>; MAX_BUS * MAX_DEVICE * MAX_FUNCTION]> = Mutex::new([None; MAX_BUS * MAX_DEVICE * MAX_FUNCTION]);
}

/// Contains PCI device properties.
#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct PCIDevice {
    /// Public vendor ID of device
    pub vendor: u32,
    /// Device number
    pub device: u32,
    pub slot: u32,
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
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct GeneralDeviceTable {
    // Address of all six BARs.
    pub bars: [u64; 6],
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
#[repr(C)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PCIToPCIBridgeTable {
    pub bars: [u64; 2],
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
    pub io_limit_upper16: u16,
    pub io_base_upper16: u16,
    pub caps_ptr: u32,
    pub expansion_rom_addr: u32,
    pub bridge_control: u32,
    pub interrupt_pin: u32,
    pub interrupt_line: u32,
}

/// This table is applicable if the Header Type is 02h (PCI-to-CardBus bridge)
#[repr(C)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
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
    let mut devices = PCI_DEVICES.lock();
    if devices[calculate_pos(
        device.bus as usize,
        device.slot as usize,
        device.func as usize,
    )]
    .is_none()
    {
        devices[calculate_pos(
            device.bus as usize,
            device.slot as usize,
            device.func as usize,
        )] = Some(device);
    } else {
        printkln!("Warning: PCI device conflict found");
        printkln!(
            "Warning: bus {:X}, slot {:X}, function {:X} already contains a device profile",
            device.bus,
            device.slot,
            device.func
        );
        for bus in 0..MAX_BUS + 1 {
            for slot in 0..MAX_DEVICE {
                for function in 0..MAX_FUNCTION {
                    if devices[calculate_pos(bus, slot, function)].is_none() {
                        printkln!("Warning: device with bus {:X}, slot {:X}, func {:X} added to PCI DB in bus {:X}, slot {:X}, func {:X}", device.bus, device.slot, device.func, bus, slot, function);
                        devices[calculate_pos(bus, slot, function)] = Some(device);
                        return;
                    }
                }
            }
        }
        panic!("Cannot add device to PCI database; bus={:X}, slot={:X}, function={:X}, vendor={:X}, class={:X}, subclass={:X}, interface={:X}", device.bus, device.slot, device.func, device.vendor, device.class, device.subclass, device.prog_if);
    }
}

/// Reads a dword from a PCI bus, device and function using the given offset and returns it.
#[no_mangle]
pub extern "C" fn read_dword(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let lbus = bus as u32;
    let lslot = slot as u32;
    let lfunc = func as u32;
    unsafe {
        outl(
            ((((lbus as u32) << 16) as u32)
                | (((lslot as u32) << 11) as u32)
                | (((lfunc as u32) << 8) as u32)
                | ((offset as u32) & 0xfc)
                | (0x80000000)) as u32,
            0xCF8,
        );
        inl(0xCFC)
    }
}

/// Writes a dword to the PCI bus
#[no_mangle]
pub extern "C" fn write_dword(bus: u8, slot: u8, func: u8, offset: u8, data: u32) {
    let lbus = bus as u32;
    let lslot = slot as u32;
    let lfunc = func as u32;
    unsafe {
        outl(
            ((((lbus as u32) << 16) as u32)
                | (((lslot as u32) << 11) as u32)
                | (((lfunc as u32) << 8) as u32)
                | ((offset as u32) & 0xfc)
                | (0x80000000)) as u32,
            0xCF8,
        );
        outl(data, 0xCFC);
    }
}

/// Reads a word from a PCI bus, device and function using the given offset and returns it.
#[no_mangle]
pub extern "C" fn read_word(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    let lbus = bus as u32;
    let lslot = slot as u32;
    let lfunc = func as u32;
    unsafe {
        outl(
            ((((lbus as u32) << 16) as u32)
                | (((lslot as u32) << 11) as u32)
                | (((lfunc as u32) << 8) as u32)
                | ((offset as u32) & 0xfc)
                | (0x80000000)) as u32,
            0xCF8,
        );
        inw(0xCFC)
    }
}

/// Writes a word to the PCI bus
#[no_mangle]
pub extern "C" fn write_word(bus: u8, slot: u8, func: u8, offset: u8, data: u16) {
    let lbus = bus as u32;
    let lslot = slot as u32;
    let lfunc = func as u32;
    unsafe {
        outl(
            ((((lbus as u32) << 16) as u32)
                | (((lslot as u32) << 11) as u32)
                | (((lfunc as u32) << 8) as u32)
                | ((offset as u32) & 0xfc)
                | (0x80000000)) as u32,
            0xCF8,
        );
        outw(data, 0xCFC);
    }
}

/// Reads a byte from a PCI bus, device and function using the given offset and returns it.
#[no_mangle]
pub extern "C" fn read_byte(bus: u8, slot: u8, func: u8, offset: u8) -> u8 {
    let lbus = bus as u32;
    let lslot = slot as u32;
    let lfunc = func as u32;
    unsafe {
        outl(
            ((((lbus as u32) << 16) as u32)
                | (((lslot as u32) << 11) as u32)
                | (((lfunc as u32) << 8) as u32)
                | ((offset as u32) & 0xfc)
                | (0x80000000)) as u32,
            0xCF8,
        );
        inb(0xCFC)
    }
}

/// Writes a byte to the PCI bus
#[no_mangle]
pub extern "C" fn write_byte(bus: u8, slot: u8, func: u8, offset: u8, data: u8) {
    let lbus = bus as u32;
    let lslot = slot as u32;
    let lfunc = func as u32;
    unsafe {
        outl(
            ((((lbus as u32) << 16) as u32)
                | (((lslot as u32) << 11) as u32)
                | (((lfunc as u32) << 8) as u32)
                | ((offset as u32) & 0xfc)
                | (0x80000000)) as u32,
            0xCF8,
        );
        outb(data, 0xCFC);
    }
}

// Here there be internals.

fn get_vendor_id(bus: u8, device: u8, function: u8) -> u32 {
    read_dword(bus, device, function, 0).get_bits(0..=15)
}

fn get_device_id(bus: u8, device: u8, function: u8) -> u32 {
    read_dword(bus, device, function, 0).get_bits(16..=31)
}

fn get_class_id(bus: u8, device: u8, function: u8) -> u32 {
    read_dword(bus, device, function, 0x08).get_bits(24..=31)
}

fn get_prog_if(bus: u8, device: u8, function: u8) -> u32 {
    read_dword(bus, device, function, 0x08).get_bits(8..=15)
}

fn get_header_type(bus: u8, device: u8, function: u8) -> u32 {
    read_dword(bus, device, function, 0x0C).get_bits(16..=23)
}

fn get_subclass_id(bus: u8, device: u8, function: u8) -> u32 {
    read_dword(bus, device, function, 0x08).get_bits(16..=23)
}

fn get_status(bus: u8, device: u8, function: u8) -> u32 {
    read_dword(bus, device, function, 0x04).get_bits(24..=31)
}

fn get_command(bus: u8, device: u8, function: u8) -> u32 {
    read_dword(bus, device, function, 0x04).get_bits(8..=15)
}

fn get_rev(bus: u8, device: u8, function: u8) -> u32 {
    read_dword(bus, device, function, 0x08).get_bits(0..=7)
}

pub fn probe() {
    printkln!("Starting PCI scan");
    for bus in 0..MAX_BUS + 1 {
        for slot in 0..MAX_DEVICE {
            for function in 0..MAX_FUNCTION {
                // Get vendor, device, class and subclass codes.
                let vendor = get_vendor_id(bus as u8, slot as u8, function as u8);
                if vendor == 0xFFFF {
                    continue;
                }
                let device = get_device_id(bus as u8, slot as u8, function as u8);
                if device == 0xFFFF {
                    continue;
                }
                let class = get_class_id(bus as u8, slot as u8, function as u8);
                let subclass = get_subclass_id(bus as u8, slot as u8, function as u8);
                // This part is the longest part of this function thus far. Here we construct the PCI device structure and its linked structures, if applicable.
                // Construction happens in this order:
                // 1. Initialize static (easily calculable/readable) data.
                // 2. Use "conditional initialization" to initialize dynamic data that requires extra reads.
                // Conditional initialization is the term I use when I take advantage of conditional statements being expressions and "conditionally" initialize parts of (or entire) data structures with them, as I do here.
                let mut pcidev = PCIDevice {
                    // Non-conditional initialization.
                    vendor: vendor,
                    device: device,
                    slot: slot as u32,
                    func: function as u32,
                    bus: bus as u32,
                    status: get_status(bus as u8, slot as u8, function as u8),
                    command: get_command(bus as u8, slot as u8, function as u8),
                    class: class,
                    subclass: subclass,
                    prog_if: get_prog_if(bus as u8, slot as u8, function as u8),
                    revision_id: get_rev(bus as u8, slot as u8, function as u8),
                    bist: read_dword(bus as u8, slot as u8, function as u8, 0x0C).get_bits(24..=31),
                    header_type: get_header_type(bus as u8, slot as u8, function as u8),
                    latency_timer: read_dword(bus as u8, slot as u8, function as u8, 0x0C)
                        .get_bits(8..=15),
                    cache_line_size: read_dword(bus as u8, slot as u8, function as u8, 0x0C)
                        .get_bits(0..=7),
                    // Determine header type and set up appropriate structures from there
                    // Conditional initialization.
                    gen_dev_tbl: if read_dword(bus as u8, slot as u8, function as u8, 0x0C)
                        .get_bits(16..=23)
                        == 0x00
                    {
                        Some(GeneralDeviceTable {
                            bars: [
                                calculate_bar_addr(
                                    read_dword(bus as u8, slot as u8, function as u8, 0x10),
                                    read_dword(bus as u8, slot as u8, function as u8, 0x14),
                                ),
                                calculate_bar_addr(
                                    read_dword(bus as u8, slot as u8, function as u8, 0x14),
                                    read_dword(bus as u8, slot as u8, function as u8, 0x18),
                                ),
                                calculate_bar_addr(
                                    read_dword(bus as u8, slot as u8, function as u8, 0x18),
                                    read_dword(bus as u8, slot as u8, function as u8, 0x1C),
                                ),
                                calculate_bar_addr(
                                    read_dword(bus as u8, slot as u8, function as u8, 0x1C),
                                    read_dword(bus as u8, slot as u8, function as u8, 0x20),
                                ),
                                calculate_bar_addr(
                                    read_dword(bus as u8, slot as u8, function as u8, 0x20),
                                    read_dword(bus as u8, slot as u8, function as u8, 0x24),
                                ),
                                calculate_bar_addr(
                                    read_dword(bus as u8, slot as u8, function as u8, 0x24),
                                    0,
                                ),
                            ],
                            cis_ptr: read_dword(bus as u8, slot as u8, function as u8, 0x28)
                                .get_bits(24..=31),
                            subsystem_id: read_dword(bus as u8, slot as u8, function as u8, 0x2C)
                                .get_bits(24..=31),
                            subsystem_vendor_id: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x2C,
                            )
                            .get_bits(16..=23),
                            expansion_rom_addr: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x30,
                            )
                            .get_bits(24..=31),
                            caps_ptr: read_dword(bus as u8, slot as u8, function as u8, 0x34)
                                .get_bits(16..=23),
                            max_latency: read_dword(bus as u8, slot as u8, function as u8, 0x3C)
                                .get_bits(24..=31),
                            min_grant: read_dword(bus as u8, slot as u8, function as u8, 0x3C)
                                .get_bits(16..=23),
                            interrupt_pin: read_dword(bus as u8, slot as u8, function as u8, 0x3C)
                                .get_bits(8..=15),
                            interrupt_line: read_dword(bus as u8, slot as u8, function as u8, 0x3C)
                                .get_bits(0..=7),
                        })
                    } else {
                        None
                    },
                    pci_to_pci_bridge_tbl: if read_dword(
                        bus as u8,
                        slot as u8,
                        function as u8,
                        0x0C,
                    )
                    .get_bits(16..=23)
                        == 0x01
                    {
                        Some(PCIToPCIBridgeTable {
                            bars: [
                                calculate_bar_addr(
                                    read_dword(bus as u8, slot as u8, function as u8, 0x10),
                                    read_dword(bus as u8, slot as u8, function as u8, 0x14),
                                ),
                                calculate_bar_addr(
                                    read_dword(bus as u8, slot as u8, function as u8, 0x14),
                                    0,
                                ),
                            ],
                            sec_latency_timer: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x18,
                            )
                            .get_bits(24..=31),
                            sub_bus: read_dword(bus as u8, slot as u8, function as u8, 0x18)
                                .get_bits(16..=23),
                            sec_bus: read_dword(bus as u8, slot as u8, function as u8, 0x18)
                                .get_bits(8..=15),
                            prim_bus: read_dword(bus as u8, slot as u8, function as u8, 0x18)
                                .get_bits(0..=7),
                            sec_status: read_dword(bus as u8, slot as u8, function as u8, 0x1C)
                                .get_bits(24..=31),
                            io_limit: read_dword(bus as u8, slot as u8, function as u8, 0x1C)
                                .get_bits(16..=23),
                            io_base: read_dword(bus as u8, slot as u8, function as u8, 0x1C)
                                .get_bits(8..=15),
                            mem_limit: read_dword(bus as u8, slot as u8, function as u8, 0x20)
                                .get_bits(24..=31),
                            mem_base: read_dword(bus as u8, slot as u8, function as u8, 0x20)
                                .get_bits(16..=23),
                            prefetch_mem_limit: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x24,
                            )
                            .get_bits(24..=31),
                            prefetch_mem_base: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x24,
                            )
                            .get_bits(16..=23),
                            prefetch_base_upper32: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x28,
                            )
                            .get_bits(0..=31),
                            prefetch_limit_upper32: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x2C,
                            )
                            .get_bits(0..=31),
                            io_limit_upper16: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x30,
                            )
                            .get_bits(16..=31) as u16,
                            io_base_upper16: read_dword(bus as u8, slot as u8, function as u8, 0x30)
                                .get_bits(0..=15)
                                as u16,
                            caps_ptr: read_dword(bus as u8, slot as u8, function as u8, 0x34)
                                .get_bits(16..=23),
                            expansion_rom_addr: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x38,
                            )
                            .get_bits(24..=31),
                            bridge_control: read_dword(bus as u8, slot as u8, function as u8, 0x3C)
                                .get_bits(24..=31),
                            interrupt_pin: read_dword(bus as u8, slot as u8, function as u8, 0x3C)
                                .get_bits(16..=23),
                            interrupt_line: read_dword(bus as u8, slot as u8, function as u8, 0x3C)
                                .get_bits(8..=15),
                        })
                    } else {
                        None
                    },
                    pci_to_card_bus_bridge_tbl: if read_dword(
                        bus as u8,
                        slot as u8,
                        function as u8,
                        0x0C,
                    )
                    .get_bits(16..=23)
                        == 0x02
                    {
                        Some(PCIToCardBusBridgeTable {
                            exca_base_addr: read_dword(bus as u8, slot as u8, function as u8, 0x10)
                                .get_bits(24..=31),
                            sec_status: read_dword(bus as u8, slot as u8, function as u8, 0x14)
                                .get_bits(24..=31),
                            caps_lst_offset: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x14,
                            )
                            .get_bits(8..=15),
                            card_bus_latency_timer: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x18,
                            )
                            .get_bits(24..=31),
                            sub_bus: read_dword(bus as u8, slot as u8, function as u8, 0x18)
                                .get_bits(16..=23),
                            card_bus_bus: read_dword(bus as u8, slot as u8, function as u8, 0x18)
                                .get_bits(8..=15),
                            pci_bus: read_dword(bus as u8, slot as u8, function as u8, 0x18)
                                .get_bits(0..=7),
                            mem_base_addr0: read_dword(bus as u8, slot as u8, function as u8, 0x1C)
                                .get_bits(24..=31),
                            mem_limit0: read_dword(bus as u8, slot as u8, function as u8, 0x20)
                                .get_bits(24..=31),
                            mem_base_addr1: read_dword(bus as u8, slot as u8, function as u8, 0x24)
                                .get_bits(24..=31),
                            mem_limit1: read_dword(bus as u8, slot as u8, function as u8, 0x28)
                                .get_bits(24..=31),
                            io_base_addr0: read_dword(bus as u8, slot as u8, function as u8, 0x2C)
                                .get_bits(24..=31),
                            io_base_limit0: read_dword(bus as u8, slot as u8, function as u8, 0x30)
                                .get_bits(24..=31),
                            io_base_addr1: read_dword(bus as u8, slot as u8, function as u8, 0x34)
                                .get_bits(24..=31),
                            io_base_limit1: read_dword(bus as u8, slot as u8, function as u8, 0x38)
                                .get_bits(24..=31),
                            bridge_control: read_dword(bus as u8, slot as u8, function as u8, 0x3C)
                                .get_bits(24..=31),
                            interrupt_pin: read_dword(bus as u8, slot as u8, function as u8, 0x3C)
                                .get_bits(16..=23),
                            interrupt_line: read_dword(bus as u8, slot as u8, function as u8, 0x3C)
                                .get_bits(8..=15),
                            subsystem_vendor_id: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x40,
                            )
                            .get_bits(24..=31),
                            subsystem_device_id: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x40,
                            )
                            .get_bits(16..=23),
                            legacy_base_addr: read_dword(
                                bus as u8,
                                slot as u8,
                                function as u8,
                                0x44,
                            )
                            .get_bits(24..=31),
                        })
                    } else {
                        None
                    },
                };
                {
                    let mut bist = read_dword(bus as u8, slot as u8, function as u8, 0x0C)
                        .get_bits(24..=31) as u8;
                    // Calculate amount of time to wait
                    let time_to_wait = {
                        let ttw = 20.0 * (1000.0 / (1000000.0 / ((32768 >> 2) as f64)));
                        ttw as u128
                    };
                    if bist.get_bit(7) {
                        bist.set_bit(6, true);
                        write_byte(bus as u8, slot as u8, function as u8, 0x0C, bist);
                        let end = get_tick_count() + time_to_wait;
                        while read_dword(bus as u8, slot as u8, function as u8, 0x0C)
                            .get_bits(24..=31)
                            .get_bit(6)
                        {
                            if get_tick_count() >= end {
                                printkln!("PCI: Bist failed on bus {:X}h, slot {:X}h, function {:X}h, device {:X}h, vendor {:X}h, class {:X}h, subclass {:X}h", bus as u8, slot as u8, function as u8, device, vendor, class, subclass);
                                continue;
                            }
                        }
                        if read_dword(bus as u8, slot as u8, function as u8, 0x0C)
                            .get_bits(24..=31)
                            .get_bits(0..=3)
                            > 0
                        {
                            printkln!("PCI: Bist failed on bus {:X}h, slot {:X}h, function {:X}h, device {:X}h, vendor {:X}h, class {:X}h, subclass {:X}h", bus as u8, slot as u8, function as u8, device, vendor, class, subclass);
                            continue;
                        }
                        pcidev.bist = read_dword(bus as u8, slot as u8, function as u8, 0x0C)
                            .get_bits(24..=31);
                    }
                }
                add_device(pcidev);
            }
        }
    }
    printkln!("Done");
}

pub fn init() {
    probe();
}

fn calculate_bar_addr(bar1: u32, bar2: u32) -> u64 {
    if !bar1.get_bit(0) {
        match bar1.get_bits(1..=2) {
            0 => ((bar1 as u64) & 0xFFFFFFF0),
            1 => ((bar1 as u64) & 0xFFF0),
            2 => (((bar1 as u64) & 0xFFFFFFF0) + (((bar2 as u64) & 0xFFFFFFFF) << 32)),
            _ => bar1 as u64,
        }
    } else {
        ((bar1 as u64) & 0xFFFFFFFC)
    }
}

#[no_mangle]
pub extern "C" fn find_device(class: u32, subclass: u32, interface: u32) -> Option<PCIDevice> {
    let devices = PCI_DEVICES.lock();
    for dev in devices.iter() {
        if dev.is_some()
            && dev.unwrap().class == class
            && dev.unwrap().subclass == subclass
            && dev.unwrap().prog_if == interface
        {
            return *dev;
        }
    }
    None
}

#[no_mangle]
pub extern "C" fn find_device_ex(
    class: u32,
    subclass: u32,
    vendors: &[u32],
    device_ids: &[u32],
) -> Option<PCIDevice> {
    let devices = PCI_DEVICES.lock();
    for dev in devices.iter() {
        if dev.is_some() && dev.unwrap().class == class && dev.unwrap().subclass == subclass {
            for it in vendors.iter().zip(device_ids.iter()) {
                let (vendor, device) = it;
                if dev.unwrap().vendor == *vendor && dev.unwrap().device == *device {
                    return *dev;
                }
            }
        }
    }
    None
}

#[inline]
fn calculate_pos(x: usize, y: usize, z: usize) -> usize {
    x * MAX_DEVICE * MAX_FUNCTION + y * MAX_FUNCTION + z
}
