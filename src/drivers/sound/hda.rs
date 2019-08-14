use crate::memory::*;
use crate::pci;
use crate::printkln;
use bit_field::BitField;
use core::ptr::*;

#[repr(u16)]
#[derive(Eq, PartialEq)]
pub enum HDARegister {
    Gcap = 0x00,
    Vmin = 0x02,
    Vmaj = 0x03,
    Outpay = 0x04,
    Inpay = 0x06,
    Gctl = 0x08,
    Wakeen = 0x0C,
    Statests = 0x0A,
    Gsts = 0x10,
    Outstrmpay = 0x18,
    Instrmpay = 0x1A,
    Intctl = 0x20,
    Intsts = 0x24,
    WallClockCounter = 0x30,
    Ssync = 0x34,
    Corblbase = 0x40,
    Corbubase = 0x44,
    Corbwp = 0x48,
    Corbrp = 0x4A,
    Corbctl = 0x4C,
    Corbsts = 0x4D,
    Corbsize = 0x4E,
    Rirblbase = 0x50,
    Rirbubase = 0x54,
    Rirbwp = 0x58,
    Rintcnt = 0x5A,
    Rirbctl = 0x5C,
    Rirbsts = 0x5D,
    Rirbsize = 0x5E,
    Dplbase = 0x70,
    Dpubase = 0x74,
    IobSDnCTL = 0x80,
    IobSD0STS = 0x83,
    IobSDnLPIB = 0x84,
    IobSDnCBL = 0x88,
    IobISDnLVI = 0x8C,
    IobSDnFIFOS = 0x90,
    IobSDnFMT = 0x92,
    IobSDnBDPL = 0x98,
    IobSDnBDPU = 0x9C,
    ImmCmdOut = 0x60,
    ImmCmdIn = 0x64,
    ImmCmdSts = 0x68,
}

pub fn init() {
    let mut memaddr: u64 = 0;
    for dev in pci::get_devices() {
        if dev.device == 0x2668 && dev.vendor == 0x8086 {
            if dev.header_type == 0 {
                let tbl = dev.gen_dev_tbl.unwrap();
                if tbl.bars[0] != 0 {
                    memaddr = tbl.bars[0];
                } else if tbl.bars[1] != 0 {
                    memaddr = tbl.bars[1];
                } else if tbl.bars[2] != 0 {
                    memaddr = tbl.bars[2];
                } else if tbl.bars[3] != 0 {
                    memaddr = tbl.bars[3];
                } else if tbl.bars[4] != 0 {
                    memaddr = tbl.bars[4];
                } else if tbl.bars[5] != 0 {
                    memaddr = tbl.bars[5];
                }
            } else if dev.header_type == 1 {
                let tbl = dev.pci_to_pci_bridge_tbl.unwrap();
                if tbl.bars[0] != 0 {
                    memaddr = tbl.bars[0];
                } else if tbl.bars[1] != 0 {
                    memaddr = tbl.bars[1];
                }
            }
            break;
        }
    }
    if memaddr != 0 {
        allocate_phys_range(memaddr, memaddr + 0x9C);
        let gcap = read_memory(memaddr);
        printkln!(
            "HDA: OSS: {}, ISS: {}, BSS: {}, NSDO: {}",
            gcap.get_bits(12..=15),
            gcap.get_bits(8..=11),
            gcap.get_bits(3..=7),
            gcap.get_bits(1..=2)
        );
        if gcap.get_bit(0) {
            printkln!("HDA: 64-bit addresses are supported for this device.");
        } else {
            printkln!("HDA: warning: 64-bit addresses are not supported by this device.");
        }
    }
}

fn read_memory(address: u64) -> u64 {
    let addr: *const u64 = address as *const u64;
    unsafe { read_volatile(addr) }
}

fn write_memory(address: u64, value: u64) {
    let addr: *mut u64 = address as *mut u64;
    unsafe {
        write_volatile(addr, value);
    }
}
