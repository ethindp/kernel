extern crate alloc;
use crate::memory::*;
use crate::pci;
use crate::printkln;
use alloc::vec::Vec;
use bit_field::BitField;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::hlt;

lazy_static! {
// AHCISUP: AHCI supported. Set to true to allow AHCI functionality (must be detected on PCI bus). Set to false to disable.
static ref AHCISUPP: Mutex<bool> = Mutex::new(false);
// BARIDX: BAR index for MMIO operations. Set to usize::max_value() to indicate unset/unknown.
static ref BAR: Mutex<u64> = Mutex::new(u64::max_value());
// PCIDEV: PCI device for internal use.
static ref PCIDEV: Mutex<pci::PCIDevice> = Mutex::new(pci::PCIDevice::default());
// HBAMEM: holds the HBA memory structures
static ref HBAMEM: Mutex<Vec<HBAMemory>> = Mutex::new(Vec::new());
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

// SATA/ATA signatures
const SIGSATA: u64 = 0x00000101; // SATA drive
const SIGATAPI: u64 = 0xEB140101; // SATAPI drive
const SIGSEM: u64 = 0xC33C0101; // Enclosure management bridge
const SIGPM: u64 = 0x96690101; // Port multiplier

#[repr(packed)]
#[derive(Copy, Clone)]
pub struct HBAMemory {
    pub cap: u32,
    pub ghc: u32,
    pub is: u32,
    pub pi: u32,
    pub vs: u32,
    pub ccc_ctl: u32,
    pub ccc_pts: u32,
    pub em_loc: u32,
    pub em_ctl: u32,
    pub cap2: u32,
    pub bohc: u32,
    rsv: [u8; 0xA0 - 0x2C],
    pub vendor: [u8; 0x100 - 0xA0],
    pub ports: [HBAPort; 32],
}

impl Default for HBAMemory {
    fn default() -> Self {
        HBAMemory {
            cap: 0,
            ghc: 0,
            is: 0,
            pi: 0,
            vs: 0,
            ccc_ctl: 0,
            ccc_pts: 0,
            em_loc: 0,
            em_ctl: 0,
            cap2: 0,
            bohc: 0,
            rsv: [0; 0xA0 - 0x2C],
            vendor: [0; 0x100 - 0xA0],
            ports: [HBAPort {
                clb: 0,
                clbu: 0,
                fb: 0,
                fbu: 0,
                is: 0,
                ie: 0,
                cmd: 0,
                rsv0: 0,
                tfd: 0,
                sig: 0,
                ssts: 0,
                sctl: 0,
                serr: 0,
                sact: 0,
                ci: 0,
                sntf: 0,
                fbs: 0,
                rsv1: [0; 11],
                vendor: [0; 4],
            }; 32],
        }
    }
}

#[repr(packed)]
#[derive(Copy, Clone, Default)]
pub struct HBAPort {
    pub clb: u32,
    pub clbu: u32,
    pub fb: u32,
    pub fbu: u32,
    pub is: u32,
    pub ie: u32,
    pub cmd: u32,
    rsv0: u32,
    pub tfd: u32,
    pub sig: u32,
    pub ssts: u32,
    pub sctl: u32,
    pub serr: u32,
    pub sact: u32,
    pub ci: u32,
    pub sntf: u32,
    pub fbs: u32,
    rsv1: [u32; 11],
    pub vendor: [u32; 4],
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct HBAFIS {
    pub dsfis: [u8; 0x1C],
    res1: [u8; 0x04],
    pub psfis: [u8; 0x14],
    res2: [u8; 0x0C],
    pub rfis: [u8; 0x14],
    res3: [u8; 0x04],
    pub sdbfis: [u8; 0x08],
    pub ufis: [u8; 0x40],
    res4: [u8; 0x60],
}

#[repr(packed)]
pub struct CommandHeader {
    pub cfl: u16,
    pub a: u8,
    pub w: u8,
    pub p: u8,
    pub r: u8,
    pub b: u8,
    pub c: u8,
    rsv0: u8,
    pub pmp: u16,
    pub prdtl: u16,
    pub prdbc: Volatile<u32>,
    pub ctba: u32,
    pub ctbau: u32,
    rsv1: [u32; 4],
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct HBACommandTable {
    pub cfis: [u8; 64],
    pub acmd: [u8; 16],
    rsv: [u8; 48],
    pub prdt_entry: [PRDTEntry; 65535],
    pub prdt_entry_cnt: u16,
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct PRDTEntry {
    pub dba: u32,
    pub dbau: u32,
    rsv0: u32,
    pub dbc: u32,
    rsv1: u32,
    pub i: u32,
}

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
            if bars[5] != 0 && !bars[5].get_bit(0) {
                if bars[5].get_bits(1..=2) == 1 {
                    panic!("AHCI: AHCI driver has 16-bit BAR address.");
                }
                allocate_phys_range(bars[5], bars[5] + BOHC);
                printkln!("AHCI: detected base address for AHCI driver: {:X}", bars[5]);
                {
                    let mut bar = BAR.lock();
                    let mut ahcisupp = AHCISUPP.lock();
                    *bar = bars[5];
                    *ahcisupp = true;
                }
                let mut mem = HBAMEM.lock();
                mem.push(HBAMemory {
                    cap: read_memory(bars[5] + CAP) as u32,
                    ghc: read_memory(bars[5] + GHC) as u32,
                    is: read_memory(bars[5] + IS) as u32,
                    pi: read_memory(bars[5] + PI) as u32,
                    vs: read_memory(bars[5] + VS) as u32,
                    ccc_ctl: read_memory(bars[5] + CCC_CTL) as u32,
                    ccc_pts: read_memory(bars[5] + CCC_PORTS) as u32,
                    em_loc: read_memory(bars[5] + EM_LOC) as u32,
                    em_ctl: read_memory(bars[5] + EM_CTL) as u32,
                    cap2: read_memory(bars[5] + CAP2) as u32,
                    bohc: read_memory(bars[5] + BOHC) as u32,
                    ports: [HBAPort {
                        clb: 0,
                        clbu: 0,
                        fb: 0,
                        fbu: 0,
                        is: 0,
                        ie: 0,
                        cmd: 0,
                        rsv0: 0,
                        tfd: 0,
                        sig: 0,
                        ssts: 0,
                        sctl: 0,
                        serr: 0,
                        sact: 0,
                        ci: 0,
                        sntf: 0,
                        fbs: 0,
                        rsv1: [0; 11],
                        vendor: [0; 4],
                    }; 32],
                    rsv: [0; 0xA0 - 0x2C],
                    vendor: [0; 0x100 - 0xA0],
                });
                let cap = read_memory(bars[5]);
                let cap2 = read_memory(bars[5] + CAP2);
                let pi = read_memory(bars[5] + PI);
                for port in 0..=read_memory(bars[5] + CAP).get_bits(0..=4) {
                    if pi.get_bit(port as usize) {
                        printkln!("AHCI: detected AHCI port {}; activating", port);
                        let portaddr: u64 = bars[5] + 0x100 + (port * 0x80);
                        let mut command: u64 = 0;
                        command.set_bits(28..=31, 1);
                        if cap.get_bit(26) {
                            command.set_bit(27, true);
                            command.set_bit(26, true);
                        }
                        command.set_bit(25, false);
                        command.set_bit(24, false);
                        if cap2.get_bit(2) {
                            command.set_bit(23, true);
                        }
                        write_memory(portaddr + PXCMD, command);
                        while read_memory(portaddr + PXCMD).get_bits(28..=31) == command {
                            hlt();
                            continue;
                        }
                        let length = mem.len();
                        mem[length - 1].ports[port as usize] = HBAPort {
                            clb: read_memory(portaddr + PXCLB) as u32,
                            clbu: read_memory(portaddr + PXCLBU) as u32,
                            fb: read_memory(portaddr + PXFB) as u32,
                            fbu: read_memory(portaddr + PXFBU) as u32,
                            is: read_memory(portaddr + PXIS) as u32,
                            ie: read_memory(portaddr + PXIE) as u32,
                            cmd: read_memory(portaddr + PXCMD) as u32,
                            tfd: read_memory(portaddr + PXTFD) as u32,
                            sig: read_memory(portaddr + PXSIG) as u32,
                            ssts: read_memory(portaddr + PXSSTS) as u32,
                            sctl: read_memory(portaddr + PXSCTL) as u32,
                            serr: read_memory(portaddr + PXSERR) as u32,
                            sact: read_memory(portaddr + PXSACT) as u32,
                            ci: read_memory(portaddr + PXCI) as u32,
                            sntf: read_memory(portaddr + PXSNTF) as u32,
                            fbs: read_memory(portaddr + PXFBS) as u32,
                            rsv0: 0,
                            rsv1: [0; 11],
                            vendor: [0; 4],
                        };
                    }
                }
                return;
            }
        }
    }
}
