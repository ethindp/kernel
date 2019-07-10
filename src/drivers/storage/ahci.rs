	use volatile::Volatile;

#[repr(u64)]
#[derive(Eq, PartialEq)]
enum GHC {
AE = 1 << 31,
IE = 1<<1,
HR = 1,
}

#[repr(u32)]
#[derive(Eq, PartialEq)]
enum Command {
St = 1,
Clo = 1 << 3,
Fre = 1 << 4,
Fr = 1 << 14,
Cr = 1 << 15,
}

const HBA_PxIS_TFES: u32 = 1 << 30;

#[repr(u64)]
#[derive(Eq, PartialEq)]
enum AhciDeviceType {
// Serial ATA (SATA) device
Sata = 0x00000101,
// Serial ATA Packet Interface (SATAPI) device
Satapi = 0xEB140101,
// Serial ATA Enclosure Management Bridge (SEMB) device
Semb = 0xC33C0101,
// Port multiplier
Pm = 0x96690101,
}

#[repr(u8)]
#[derive(Eq, PartialEq)]
enum AtaStatus {
Error = 0x01,
Drq = 0x08,
Srv = 0x10,
Df = 0x20,
Rdy = 0x40,
Bsy = 0x80,
}

const CMD_FIS_DEV_LBA: u8 = 1 << 6;
const MAX_CMD_SLOT_CNT: u8 = 32;
const MAX_PORT_CNT: u8 = 32;

#[repr(u16)]
#[derive(Eq, PartialEq)]
enum FisType {
RegH2d = 0x27,
RegD2h = 0x34,
DmaAct = 0x39,
DmaSetup = 0x41,
Data = 0x46,
Bist = 0x58,
PioSetup = 0x5F,
DevBits = 0xA1,
}


#[repr(packed)]
pub struct RegH2D {
pub fis_type: &'static FisType,
pub pm_port: u8,
pub rsv0: u8,
pub c: u8,
pub command: u8,
pub featurel: u8,
pub lba0: u8,
pub lba1: u8,
pub lba2: u8,
pub device: u8,
pub lba3: u8,
pub lba4: u8,
pub lba5: u8,
pub featureh: u8,
pub count: u16,
pub icc: u8,
pub control: u8,
pub aux: u16,
pub rsv1: [u8; 2],
}

#[repr(packed)]
pub struct RegD2H {
pub fis_type: &'static FisType,
pub pm_port: u8,
pub rsv0: u8,
pub i: u8,
pub rsv1: u8,
pub status: u8,
pub error: u8,
pub lba0: u8,
pub lba1: u8,
pub lba2: u8,
pub device: u8,
pub lba3: u8,
pub lba4: u8,
pub lba5: u8,
pub rsv2: u8,
pub count: u16,
pub rsv3: [u8; 2],
pub rsv4: [u8; 4],
}

#[repr(packed)]
pub struct DevBits {
pub fis_type: &'static FisType,
pub pm_port: u8,
pub rsv0: u8,
pub i: u8,
pub n: u8,
pub statusl: u8,
pub rsvp1: u8,
pub statush: u8,
pub rsv2: u8,
pub error: u8,
pub protocol: u32,
}

#[repr(packed)]
pub struct DmaSetup {
pub fis_type: &'static FisType,
pub pm_port: u8,
pub rsv0: u8,
pub d: u8,
pub i: u8,
pub a: u8,
pub rsved: [u8; 2],
pub buf_id: u64,
pub rsv1: u32,
pub buf_offset: u32,
pub trans_count: u32,
pub rsv2: u32,
}

#[repr(packed)]
pub struct PioSetup {
pub fis_type: &'static FisType,
pub pm_port: u8,
pub rsv0: u8,
pub rsv1: [u8; 2],
pub data: [u32; 1],
}

#[repr(packed)]
pub struct PRDTEntry {
pub dba: u64,
pub rsv0: u32,
pub dbc: u32,
pub rsv1: u16,
pub i: u8,
}

#[repr(packed)]
#[repr(align(128))]
pub struct HbaCommandTable {
pub cfis: [u8; 64],
pub acmd: [u8; 16],
pub rsv: [u8; 48],
pub prdt_entry: &'static PRDTEntry,
}

#[repr(packed)]
pub struct HbaHeader {
pub cfl: u8,
pub a: u8,
pub w: u8,
pub p: u8,
pub r: u8,
pub b: u8,
pub c: u8,
pub rsv0: u8,
pub pmp: u8,
pub prdtl: u16,
pub prdbc: Volatile<u32>,
pub ctba: u64,
pub rsv1: [u32; 4],
}

#[repr(packed)]
#[repr(align(256))]
pub struct HbaFis {
pub dsfis: &'static DmaSetup,
pub pad0: [u8; 4],
pub psfis: &'static PioSetup,
pub pad1: [u8; 12],
pub rfis: &'static RegD2H,
pub pad2: [u8; 4],
pub sdbfis: &'static DevBits,
pub ufis: [u8; 64],
pub rsv: [u8; 96],
}

#[repr(packed)]
#[repr(align(1024))]
pub struct HbaCommandList {
pub headers: [&'static HbaCommandHeader; MAX_CMD_SLOT_CNT],
}

#[repr(packed)]
pub struct HbaPort {
pub clb: u64,
pub fb: u64,
pub is_rwc: u32,
pub ie: u32,
pub cmd: u32,
pub rsv0: u32,
pub tfd: u32,
pub sig: u32,
pub ssts: u32,
pub sctl: u32,
pub serr_rwc: u32,
pub sact: u32,
pub ci: u32,
pub sntf_rwc: u32,
pub fbs: u32,
pub rsv1: [u32; 11],
pub vendor: [u32; 4],
}

#[repr(packed)]
pub struct HbaMem {
pub cap: u32,
pub ghc: u32,
pub is_rwc: u32,
pub pi: u32,
pub vs: u32,
pub ccc_ctl: u32,
pub ccc_pts: u32,
pub em_loc: u32,
pub em_ctl: u32,
pub cap2: u32,
pub bohc: u32,
pub rsv: [u8; 0xA0 - 0x2C],
pub vendor: [u8; 0x100 - 0xA0],
pub ports: [&'static HbaPort; MAX_PORT_CNT],
}

#[repr(packed)]
pub struct FisData {
pub fis_type: &'static FisType,
pub pm_port: u8,
pub rsv0: u8,
pub rsv1: [u8; 2],
pub data: [u32; 1],
}

