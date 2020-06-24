use bitflags::bitflags;

const GCAP: usize = 0x00;
const VMIN: usize = 0x02;
const VMAJ: usize = 0x03;
const GCTL: usize = 0x08;
const WAKEEN: usize = 0x0C;
const STATESTS: usize = 0x0E;
const INTCTL: usize = 0x20;
const INTSTS: usize = 0x24;
const CORBLBASE: usize = 0x40;
const CORBUBASE: usize = 0x44;
const CORBWP: usize = 0x48;
const CORBRP: usize = 0x4A;
const CORBCTL: usize = 0x4C;
const CORBSTS: usize = 0x4D;
const CORBSIZE: usize = 0x4E;
const RIRBLBASE: usize = 0x50;
const RIRBUBASE: usize = 0x54;
const RIRBWP: usize = 0x58;
const RINTCNT: usize = 0x5A;
const RIRBCTL: usize = 0x5C;
const RIRBSTS: usize = 0x5D;
const RIRBSIZE: usize = 0x5E;
const DPLBASE: usize = 0x70;
const DPUBASE: usize = 0x74;
const STRMBASE: usize = 0x80;
const STRMSIZE: usize = 0x20;
const STRMCTL0: usize = 0x00;
const STRMCTL1: usize = 0x01;
const STRMCTL2: usize = 0x02;
const STRMSTS: usize = 0x03;
const STRMLPIB: usize = 0x04;
const STRMCBL: usize = 0x08;
const STRMLVI: usize = 0x0C;
const STRMFIFOS: usize = 0x10;
const STRMFMT: usize = 0x12;
const STRMBDPL: usize = 0x18;
const STRMBDPU: usize = 0x1C;
const TCSEL: usize = 0x44;
const ATI_CNTR2: usize = 0x42;
const NVIDIA_OSTRM_COH: usize = 0x4C;
const NVIDIA_ISTRM_COH: usize = 0x4D;
const NVIDIA_TRANS: usize = 0x4E;
const INTEL_SCHDEVC: usize = 0x78;

pub fn get_streams(cap: u16) -> (u16, u16, u16) {
((cap >> 12) & 15, (cap >> 8) & 15, (cap >> 3) & 15)
}

bitflags! {
pub struct GlobalControl: u32 {
const UNSOL = 1 << 8;
const FCNTRL = 1 << 1;
const CRST = 1 << 0;
}

pub struct InterruptControl: u32 {
const GIE = 1 << 31;
const CIE = 1 << 30;
}

pub struct InterruptStatus: u32 {
const GIS = 1 << 31;
const CIS = 1 << 30;
}

pub struct CorbControl: u8 {
const RUN = 1 << 1;
const MEIE = 1 << 0;
}

pub struct RirbControl: u8 {
const ROIC = 1 << 2;
const DMAE = 1 << 1;
const INTCTL = 1 << 0;
}

pub struct RirbStatus: u8 {
const OIS = 1 << 2;
const INTFL = 1 << 0;
}

pub struct StreamControl: u32 {
const DIR = 1 << 19;
const TP = 1 << 18;
const EIE = 1 << 4;
const FEIE = 1<< 3;
const IOCE = 1 << 2;
const RUN = 1 << 1;
const RST = 1 << 0;
}

pub struct StreamStatus: u8 {
const FIFORDY = 1 << 5;
const DESE = 1 << 4;
const FIFOE = 1 << 3;
const BCIS = 1 << 2;
}

pub struct StreamFormat: u16 {
const BASE = 1 << 14;
}
}

// Masks
const WAKEEN_MASK: u32 = 0x7FFF;
const STRMSTS_MASK: u32 = 0x3FFFFFFF;
const CORBWP_MASK: u16 = 0xFF;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RirbEntry {
pub response: u32,
pub flags: u32
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BdlEntry {
pub lower: u32,
pub upper: u32,
pub length: u32,
pub ioc: u32
}

