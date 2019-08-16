use crate::memory::*;
use crate::pci;
use crate::{printk, printkln};
use bit_field::BitField;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::hlt;

lazy_static! {
// AHCISUP: AHCI supported. Set to true to allow AHCI functionality (must be detected on PCI bus). Set to false to disable.
static ref AHCISUPP: Mutex<bool> = Mutex::new(false);
// BARIDX: BAR index for MMIO operations. Set to usize::max_value() to indicate unset/unknown.
static ref BARIDX: Mutex<usize> = Mutex::new(usize::max_value());
// PCIDEV: PCI device for internal use.
static ref PCIDEV: Mutex<pci::PCIDevice> = Mutex::new(pci::PCIDevice::default());
}

// Generic HBA registers

// HBA Capabilities: This register indicates basic capabilities of the HBA to driver software.
const CAP: u64 = 0x00;
// Global HBA Control: This register controls various global actions of the HBA.
const GHC: u64 = 0x04;
// Interrupt Status Register: This register indicates which of the ports within the controller have an interrupt pending and require service.
const IS: u64 = 0x08;
// Ports Implemented: This register indicates which ports are exposed by the HBA. It is loaded by the BIOS. It indicates which
// ports that the HBA supports are available for software to use. For example, on an HBA that supports 6
// ports as indicated in CAP.NP, only ports 1 and 3 could be available, with ports 0, 2, 4, and 5 being
// unavailable.
// Software must not read or write to registers within unavailable ports.
// The intent of this register is to allow system vendors to build platforms that support less than the full
// number of ports implemented on the HBA silicon.
const PI: u64 = 0x0C;
// Version: This register indicates the major and minor version of the AHCI specification that the HBA implementation
// supports. The upper two bytes represent the major version number, and the lower two bytes represent
// the minor version number. Example: Version 3.12 would be represented as 00030102h. Three versions
// of the specification are valid: 0.95, 1.0, 1.1, 1.2, 1.3, and 1.3.1.
// VS Value for 0.95 Compliant HBAs: 0000905h
// VS Value for 1.0 Compliant HBAs: 00010000h
// VS Value for 1.1 Compliant HBAs: 00010100h
// VS Value for 1.2 Compliant HBAs: 00010200h
// VS Value for 1.3 Compliant HBAs: 00010300h
// VS Value for 1.3.1 Compliant HBAs: 00010301h
const VS: u64 = 0x10;
// command completion coalescing control register: The command completion coalescing control register is used to configure the command completion
// coalescing feature for the entire HBA.
// Implementation Note: HBA state variables (examples include hCccComplete and hCccTimer) are used
// to describe the required externally visible behavior. Implementations are not required to have internal
// state values that directly correspond to these variables.
const CCC_CTL: u64 = 0x14;
// command completion coalescing ports: The command completion coalescing ports register is used to specify the ports that are coalesced as part
// of the CCC feature when CCC_CTL.EN = '1'.
const CCC_PORTS: u64 = 0x18;
// Enclosure Management Location: The enclosure management location register identifies the location and size of the enclosure
// management message buffer.
const EM_LOC: u64 = 0x1C;
// Enclosure Management Control: This register is used to control and obtain status for the enclosure management interface. The register
// includes information on the attributes of the implementation, enclosure management messages
// supported, the status of the interface, whether any messages are pending, and is used to initiate sending
// messages.
const EM_CTL: u64 = 0x20;
// HBA Capabilities Extended: This register indicates capabilities of the HBA to driver software.
const CAP2: u64 = 0x24;
// BIOS/OS Handoff Control and Status: This register controls various global actions of the HBA. This register is not affected by an HBA reset.
const BOHC: u64 = 0x28;

// Port x Command List Base Address
const PXCLB: u64 = 0x00;
// Port x Command List Base Address Upper 32-bits
const PXCLBU: u64 = 0x04;
// Port x FIS Base Address
const PXFB: u64 = 0x08;
// Port x FIS Base Address Upper 32-Bits
const PXFBU: u64 = 0x0C;
// Port x Interrupt Status
const PXIS: u64 = 0x10;
// Port x Interrupt Enable
const PXIE: u64 = 0x14;
// Port x Command and Status
const PXCMD: u64 = 0x18;
// Port x Task File Data
const PXTFD: u64 = 0x20;
// Port x Signature
const PXSIG: u64 = 0x24;
// Port x Serial ATA Status (SCR0: SStatus)
const PXSSTS: u64 = 0x28;
// Port x Serial ATA Control (SCR2: SControl)
const PXSCTL: u64 = 0x2C;
// Port x Serial ATA Error (SCR1: SError)
const PXSERR: u64 = 0x30;
// Port x Serial ATA Active (SCR3: SActive)
const PXSACT: u64 = 0x34;
// Port x Command Issue
const PXCI: u64 = 0x38;
// Port x Serial ATA Notification (SCR4: SNotification)
const PXSNTF: u64 = 0x3C;
// Port x FIS-based Switching Control
const PXFBS: u64 = 0x40;
// Port x Device Sleep
const PXDEVSLP: u64 = 0x44;

const SIGSATA: u64 = 0x00000101; // SATA drive
const SIGATAPI: u64 = 0xEB140101; // SATAPI drive
const SIGSEM: u64 = 0xC33C0101; // Enclosure management bridge
const SIGPM: u64 = 0x96690101; // Port multiplier

pub fn init() {
    for dev in pci::get_devices() {
        if dev.class == 0x01 && dev.subclass == 0x06 && dev.prog_if == 0x01 {
            printkln!(
                "AHCI: found AHCI-capable device with vendor {:X} and device {:X}",
                dev.vendor,
                dev.device
            );
            {
                let mut pcdev = PCIDEV.lock();
                *pcdev = dev;
            }
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
            for idx in 0..=bars.len() {
                if bars[idx] != 0 && !bars[idx].get_bit(0) {
                    if bars[idx].get_bits(1..=2) == 1 {
                        panic!("AHCI: AHCI driver has 16-bit BAR address.");
                    }
                    printkln!(
                        "AHCI: detected base address for AHCI driver: {:X}",
                        bars[idx]
                    );
                    {
                        let mut bidx = BARIDX.lock();
                        let mut ahcisupp = AHCISUPP.lock();
                        *bidx = idx;
                        *ahcisupp = true;
                    }
                    let mut command: u64 = 0;
                    command.set_bit(31, true);
                    write_memory(bars[idx] + GHC, command);
                    let pi = read_memory(bars[idx] + PI);
                    for port in 0..=read_memory(bars[idx] + CAP).get_bits(0..=4) {
                        if pi.get_bit(port as usize) {
                            printkln!("AHCI: detected AHCI port {}; activating", port);
                            let portaddr: u64 = bars[idx] + 0x100 + (port * 0x80);
                            let mut command: u64 = 0;
                            command.set_bits(28..=31, 1);
                            write_memory(portaddr + PXCMD, command);
                            while read_memory(portaddr + PXCMD).get_bits(28..=31) == command {
                                hlt();
                                continue;
                            }
                        }
                    }
                    return;
                }
            }
        }
    }
}
