#[repr(packed)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct FisRegH2D {
    pub fis_type: u8,
    pub pmport: u8,
    rsv0: u8,
    pub c: u8,
    pub command: u8,
    pub feature_lo: u8,
    pub lba0: u8,
    pub lba1: u8,
    pub lba2: u8,
    pub device: u8,
    pub lba3: u8,
    pub lba4: u8,
    pub lba5: u8,
    pub feature_hi: u8,
    pub count_lo: u8,
    pub count_hi: u8,
    pub icc: u8,
    pub control: u8,
    rsv1: [u8; 4],
}

#[repr(packed)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct FisRegD2H {
    pub fis_type: u8,
    pub pmport: u8,
    rsv0: u8,
    pub i: u8,
    rsv1: u8,
    pub status: u8,
    pub error: u8,
    pub lba0: u8,
    pub lba1: u8,
    pub lba2: u8,
    pub device: u8,
    pub lba3: u8,
    pub lba4: u8,
    pub lba5: u8,
    rsv2: u8,
    pub count_lo: u8,
    pub count_hi: u8,
    rsv3: [u8; 2],
    rsv4: [u8; 4],
}

#[repr(packed)]
pub struct FisData {
    pub fis_type: u8,
    pub pmport: u8,
    rsv0: u8,
    rsv1: [u8; 2],
    pub data: [u32],
}

#[repr(packed)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct FisPioSetup {
    pub fis_type: u8,
    pub pmport: u8,
    rsv0: u8,
    pub d: u8,
    pub i: u8,
    rsv1: u8,
    pub status: u8,
    pub error: u8,
    pub lba0: u8,
    pub lba1: u8,
    pub lba2: u8,
    pub device: u8,
    pub lba3: u8,
    pub lba4: u8,
    pub lba5: u8,
    rsv2: u8,
    pub count_lo: u8,
    pub count_hi: u8,
    rsv3: u8,
    pub e_status: u8,
    pub tc: u16,
    rsv4: [u8; 2],
}

#[repr(packed)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct FisDmaSetup {
    pub fis_type: u8,
    pub pmport: u8,
    rsv0: u8,
    pub d: u8,
    pub i: u8,
    pub a: u8,
    rsved: [u8; 2usize],
    pub dma_buf_id: u64,
    rsvd: u32,
    pub dma_buf_offset: u32,
    pub transfer_count: u32,
    resvd: u32,
}

#[repr(packed)]
#[derive(Copy, Clone)]
pub struct HbaFis {
    pub dsfis: FisDmaSetup,
    pad0: [u8; 4],
    pub psfis: FisPioSetup,
    pad1: [u8; 12],
    pub rfis: FisRegD2H,
    pad2: [u8; 4],
    pub sdbfis: u16,
    pub ufis: [u8; 64],
    rsv: [u8; 96],
}

#[repr(packed)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct HbaCmdHeader {
    pub cfl: u8,
    pub a: u8,
    pub w: u8,
    pub p: u8,
    pub r: u8,
    pub b: u8,
    pub c: u8,
    rsv0: u8,
    pub pmp: u8,
    pub prdtl: u16,
    pub prdbc: u32,
    pub ctba: u32,
    pub ctbau: u32,
    rsv1: [u32; 4],
}

#[repr(packed)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct HbaPrdtEntry {
    pub dba: u32,
    pub dbau: u32,
    rsv0: u32,
    pub dbc: u32,
    rsv1: u32,
    pub i: u8,
}

#[repr(packed)]
pub struct HbaCmdTbl {
    pub cfis: FisRegH2D,
    pub acmd: [u8; 16],
    rsv: [u8; 48],
    pub prdt_entry: [HbaPrdtEntry; 1024],
}

#[repr(packed)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct HbaPort {
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
#[derive(Copy, Clone)]
pub struct HbaMem {
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
    rsv: [u8; 116],
    pub vendor: [u8; 96],
    pub ports: [HbaPort; 32],
}
