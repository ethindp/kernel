use crate::memory::*;
use crate::pci;
use crate::printkln;
use bit_field::BitField;
use crate::interrupts::sleep_for;
use x86_64::instructions::random::RdRand;
use x86_64::align_up;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::hlt;

lazy_static! {
static ref SDINS: Mutex<[bool; 15]> = Mutex::new([false; 15]);
static ref CORBADDR: Mutex<u64> = Mutex::new(0);
static ref RIRBADDR: Mutex<u64> = Mutex::new(0);
}

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
    IobSDnSTS = 0x83,
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
        if (dev.class == 0x04 && dev.subclass == 0x03)
            && (dev.vendor == 0x8086 || dev.vendor == 0x1002)
            && (dev.device == 0x2668 || dev.device == 0x27D8 || dev.device == 0x4383)
        {
                    {
            let mut command = dev.command as u16;
            command.set_bits(8 ..= 10, 1);
            command.set_bits(0 ..= 6, 1);
            pci::write_word(dev.bus as u8, dev.device as u8, dev.func as u8, 0x04, command);
            }
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
                printkln!("HDA: Interrupt pin {:X}h, interrupt line {:X}h", tbl.interrupt_pin, tbl.interrupt_line);
            } else if dev.header_type == 1 {
                let tbl = dev.pci_to_pci_bridge_tbl.unwrap();
                if tbl.bars[0] != 0 {
                    memaddr = tbl.bars[0];
                } else if tbl.bars[1] != 0 {
                    memaddr = tbl.bars[1];
                }
printkln!("HDA: Interrupt pin {:X}h, interrupt line {:X}h", tbl.interrupt_pin, tbl.interrupt_line);
            }
            break;
        }
    }
    if memaddr != 0 {
        allocate_phys_range(memaddr, memaddr + 0x9C);
        init_hda(memaddr);
    }
}

fn init_hda(memaddr: u64) {
printkln!("HDA: init: resetting HDA controller");
write_memory(memaddr + HDARegister::Statests as u64, (1 << 8) as u64);
{
let mut gctl = read_memory(memaddr + HDARegister::Gctl as u64) as u32;
gctl.set_bit(0, true);
write_memory(memaddr + HDARegister::Gctl as u64, gctl as u64);
loop {
if read_memory(memaddr + HDARegister::Gctl as u64) .get_bit(0) {
break;
}
for _ in 256 ..= 0 {
continue;
}
hlt();
}
sleep_for(10);
}
printkln!("HDA: init: reset complete");
printkln!("HDA: init: configuring HDA controller");
{
let mut wakeen = read_memory(memaddr + HDARegister::Wakeen as u64) as u16;
wakeen.set_bits(0 ..= 14, 1);
write_memory(memaddr + HDARegister::Wakeen as u64, wakeen as u64);
}
// Setup CORB
printkln!("HDA: init: corb: configuring");
{
let mut corbctl = read_memory(memaddr + HDARegister::Corbctl as u64) as u8;
corbctl.set_bit(1, false);
corbctl.set_bit(0, false);
write_memory(memaddr + HDARegister::Corbctl as u64, corbctl as u64);
}
{
let mut corbsize = read_memory(memaddr + HDARegister::Corbsize as u64) as u8;
// determine size of CORB based on bits, abort at first largest size (in order)
printkln!("HDA: init: corb: setting corb size max to 256 entries");
corbsize.set_bits(0 ..= 1, 0x02);
write_memory(memaddr + HDARegister::Corbsize as u64, corbsize as u64);
// Generate a random address
let mut addr = align_up({
let mut val = RdRand::new().unwrap().get_u64().unwrap();
if val.get_bits(48 .. 64) != 0 {
val.set_bits(48 .. 64, 0);
}
val
}, 128);
if addr.get_bits(48 .. 64) != 0 {
addr.set_bits(48 .. 64, 0);
addr = align_up(addr, 128);
}
printkln!("HDA: init: corb: allocating corb at addr {:X}h", addr);
allocate_phys_range(addr, addr + 1024);
write_memory(memaddr + HDARegister::Corblbase as u64, addr.get_bits(0 .. 32));
write_memory(memaddr + HDARegister::Corbubase as u64, addr.get_bits(32 .. 64));
}
printkln!("HDA: init: corb: clearing corb WP");
write_memory(memaddr + HDARegister::Corbwp as u64, 0);
printkln!("HDA: init: corb: resetting corb RP");
while read_memory(memaddr + HDARegister::Corbrp as u64).get_bit(15) {
let mut val = 0u64;
val.set_bit(15, true);
write_memory(memaddr + HDARegister::Corbrp as u64, val);
for _ in 256 ..= 0 {
continue;
}
}
printkln!("HDA: init: corb: configuration complete");
printkln!("HDA: init: rirb: configuring");
{
let mut rirbctl = read_memory(memaddr + HDARegister::Rirbctl as u64) as u8;
rirbctl.set_bit(1, false);
rirbctl.set_bit(0, false);
rirbctl.set_bit(2, false);
write_memory(memaddr + HDARegister::Rirbctl as u64, rirbctl as u64);
}
{
let mut rirbsize = read_memory(memaddr + HDARegister::Rirbsize as u64) as u8;
printkln!("HDA: init: rirb: setting rirb size max to 256 entries");
rirbsize.set_bits(0 ..= 1, 0x02);
write_memory(memaddr + HDARegister::Rirbsize as u64, rirbsize as u64);
// Generate a random address
let mut addr = align_up({
let mut val = RdRand::new().unwrap().get_u64().unwrap();
if val.get_bits(48 .. 64) != 0 {
val.set_bits(48 .. 64, 0);
}
val
}, 128);
if addr.get_bits(48 .. 64) != 0 {
addr.set_bits(48 .. 64, 0);
addr = align_up(addr, 128);
}
printkln!("HDA: init: rirb: allocating rirb at addr {:X}h", addr);
allocate_phys_range(addr, addr + 2048);
write_memory(memaddr + HDARegister::Rirblbase as u64, addr.get_bits(0 .. 32));
write_memory(memaddr + HDARegister::Rirbubase as u64, addr.get_bits(32 .. 64));
}
printkln!("HDA: init: rirb: configuration complete");
printkln!("HDA: init: starting corb and rirb");
{
let mut corbctl = read_memory(memaddr + HDARegister::Corbctl as u64) as u8;
corbctl.set_bit(1, true);
corbctl.set_bit(0, false);
write_memory(memaddr + HDARegister::Corbctl as u64, corbctl as u64);
}
{
let mut rirbctl = read_memory(memaddr + HDARegister::Rirbctl as u64) as u8;
rirbctl.set_bit(1, true);
rirbctl.set_bit(0, false);
rirbctl.set_bit(2, false);
write_memory(memaddr + HDARegister::Rirbctl as u64, rirbctl as u64);
}
// Verify that all is working fine
{
let corbsts = read_memory(memaddr + HDARegister::Corbsts as u64) as u8;
if corbsts.get_bit(0) {
panic!("HDA: CMEI set!");
}
}
{
let statests = read_memory(memaddr + HDARegister::Statests as u64).get_bits(0 ..= 14);
let mut sdins = SDINS.lock();
for codec in 0 ..= 14 {
sdins[codec] = statests.get_bit(codec);
if sdins[codec] {
printkln!("HDA: detected codec {}", codec);
}
}
}
}
