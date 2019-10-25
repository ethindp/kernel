mod internal;
extern crate alloc;
extern crate volatile;
use crate::memory::*;
use crate::pci;
use crate::printkln;
use alloc::vec::Vec;
use bit_field::BitField;
use core::mem::{size_of, transmute, zeroed};
use core::ptr::write_bytes;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
// HBADB: An array of up to 64 Host bus adapters (HBAs)
// Allows for up to 2,048 HBA ports
static ref HBADB: Mutex<[AhciDevice; 64]> = Mutex::new([AhciDevice {
bar: 0,
device: pci::PCIDevice::default(),
idx: 0,
}; 64]);
}

#[derive(Clone, Debug, Copy)]
pub struct AhciDevice {
    pub idx: usize,
    pub bar: u64,
    pub device: pci::PCIDevice,
}

// SATA/ATA signatures
const SIG_SATA: u64 = 0x00000101; // SATA drive
const SIG_ATAPI: u64 = 0xEB140101; // SATAPI drive
const SIG_SEM: u64 = 0xC33C0101; // Enclosure management bridge
const SIG_PM: u64 = 0x96690101; // Port multiplier

// Base address, 4M
const AHCI_BASE: u64 = 0x400000;

#[repr(u8)]
pub enum AhciDeviceType {
    Null = 0,
    Sata,
    Sem,
    Pm,
    Satapi,
}

#[repr(u8)]
pub enum HBAPortStatus {
    DetPresent = 3,
    IpmActive = 1,
}

#[repr(u16)]
pub enum PortCommand {
    Cr = 1 << 15,
    Fr = 1 << 14,
    Fre = 1 << 4,
    Sud = 1 << 1,
    St = 1 << 0,
}

#[repr(u8)]
pub enum AtaStatus {
    Busy = 0x80,
    Drq = 0x08,
}

#[repr(u32)]
pub enum AhciCommand {
    CfaEraseSectors = 0xC0,
    CfaRequestExtendedErrorCode = 0x03,
    CfaTranslateSector = 0x87,
    CfaWriteMultipleWithoutErase = 0xCD,
    CfaWriteSectorsWithoutErase = 0x38,
    CheckMediaCardType = 0xD1,
    CheckPowerMode = 0xE5,
    ConfigureStream = 0x51,
    DeviceConfigure = 0xB1,
    DeviceReset = 0x08,
    DownloadMicrocode = 0x92,
    ExecuteDeviceDiagnostic = 0x90,
    FlushCache = 0xE7,
    FlushCacheExt = 0xEA,
    IdentifyDevice = 0xEC,
    IdentifyPacketDevice = 0xA1,
    Idle = 0xE3,
    IdleImmediate = 0xE1,
    Nop = 0x00,
    NvCache = 0xB6,
    Packet = 0xA0,
    ReadBuffer = 0xE4,
    ReadDma = 0xC8,
    ReadDmaExt = 0x25,
    ReadDmaQueued = 0xC7,
    ReadDmaQueuedExt = 0x26,
    ReadFpdmaQueued = 0x60,
    ReadLogExt = 0x2F,
    ReadLogDmaExt = 0x47,
    ReadMultiple = 0xC4,
    ReadMultipleExt = 0x29,
    ReadNativeMaxAddress = 0xF8,
    ReadNativeMaxAddressExt = 0x27,
    ReadSectors = 0x20,
    ReadSectorsExt = 0x24,
    ReadStreamDmaExt = 0x2A,
    ReadStreamExt = 0x2B,
    ReadVerifySectors = 0x40,
    ReadVerifySectorsExt = 0x42,
    SecurityDisablePassword = 0xF6,
    SecurityErasePrepare = 0xF3,
    SecurityEraseUnit = 0xF4,
    SecurityFrezeLock = 0xF5,
    SecuritySetPassword = 0xF1,
    SecurityUnlock = 0xF2,
    Service = 0xA2,
    SetFeatures = 0xEF,
    SetMax = 0xF9,
    SetMaxAddressExt = 0x37,
    SetMultipleMode = 0xC6,
    Sleep = 0xE6,
    Smart = 0xB0,
    Standby = 0xE2,
    StandbyImmediate = 0xE0,
    TrustedNonData = 0x5B,
    TrustedReceive = 0x5C,
    TrustedReceiveDma = 0x5D,
    TrustedSend = 0x5E,
    TrustedSendDma = 0x5F,
    WriteBuffer = 0xE8,
    WriteDma = 0xCA,
    WriteDmaExt = 0x35,
    WriteDmaFuaExt = 0x3D,
    WriteDmaQueued = 0xCC,
    WriteDmaQueuedExt = 0x36,
    WriteDmaQueuedFuaExt = 0x3E,
    WriteFpdmaQueued = 0x61,
    WriteLogExt = 0x3F,
    WriteLogDmaExt = 0x57,
    WriteMultiple = 0xC5,
    WriteMultipleExt = 0x39,
    WriteMultipleFuaExt = 0xCE,
    WriteSectors = 0x30,
    WriteSectorsExt = 0x34,
    WriteStreamDmaExt = 0x3A,
    WriteStreamExt = 0x3B,
    WriteUncorrectableExt = 0x45,
}

#[repr(u32)]
pub enum DCOSubcommand {
    DeviceConfigurationFreezeLock = 0xC1,
    DeviceConfigurationIdentify = 0xC2,
    DeviceConfigurationRestore = 0xC0,
    DeviceConfigurationSet = 0xC3,
}

#[repr(u32)]
pub enum NvCacheSubcommand {
    AddLbasToPinnedSet = 0x10,
    Flush = 0x14,
    Disable = 0x16,
    Enable = 0x15,
    QueryMisses = 0x13,
    QueryPinnedSet = 0x12,
    RemoveLbasFromPinnedSet = 0x11,
    ReturnFromPowerMode = 0x01,
    SetPowerMode = 0x00,
}

#[repr(u8)]
pub enum FisType {
    RegH2D = 0x27,
    RegD2H = 0x34,
    DmaAct = 0x39,
    DmaSetup = 0x41,
    Data = 0x46,
    Bist = 0x58,
    PioSetup = 0x5F,
    DevBits = 0xA1,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct FisRegH2D {
    pub fis_type: cty::c_uchar,
    _bitfield_1: internal::bitfield<[u8; 1usize], u8>,
    pub command: cty::c_uchar,
    pub feature_lo: cty::c_uchar,
    pub lba0: cty::c_uchar,
    pub lba1: cty::c_uchar,
    pub lba2: cty::c_uchar,
    pub device: cty::c_uchar,
    pub lba3: cty::c_uchar,
    pub lba4: cty::c_uchar,
    pub lba5: cty::c_uchar,
    pub feature_hi: cty::c_uchar,
    pub count_lo: cty::c_uchar,
    pub count_hi: cty::c_uchar,
    pub icc: cty::c_uchar,
    pub control: cty::c_uchar,
    rsv1: [cty::c_uchar; 4usize],
}

impl FisRegH2D {
    #[inline]
    pub fn pmport(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(0usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmport(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn c(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(7usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_c(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(7usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn new_bitfield_1(
        pmport: cty::c_uchar,
        rsv0: cty::c_uchar,
        c: cty::c_uchar,
    ) -> internal::bitfield<[u8; 1usize], u8> {
        let mut bitfield: internal::bitfield<[u8; 1usize], u8> = Default::default();
        bitfield.set(0usize, 4u8, {
            let pmport: u8 = unsafe { transmute(pmport) };
            pmport as u64
        });
        bitfield.set(4usize, 3u8, {
            let rsv0: u8 = unsafe { transmute(rsv0) };
            rsv0 as u64
        });
        bitfield.set(7usize, 1u8, {
            let c: u8 = unsafe { transmute(c) };
            c as u64
        });
        bitfield
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct FisRegD2H {
    pub fis_type: cty::c_uchar,
    _bitfield_1: internal::bitfield<[u8; 1usize], u8>,
    pub status: cty::c_uchar,
    pub error: cty::c_uchar,
    pub lba0: cty::c_uchar,
    pub lba1: cty::c_uchar,
    pub lba2: cty::c_uchar,
    pub device: cty::c_uchar,
    pub lba3: cty::c_uchar,
    pub lba4: cty::c_uchar,
    pub lba5: cty::c_uchar,
    pub rsv2: cty::c_uchar,
    pub count_lo: cty::c_uchar,
    pub count_hi: cty::c_uchar,
    pub rsv3: [cty::c_uchar; 2usize],
    pub rsv4: [cty::c_uchar; 4usize],
}

impl FisRegD2H {
    #[inline]
    pub fn pmport(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(0usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmport(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn rsv0(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(4usize, 2u8) as u8) }
    }

    #[inline]
    pub fn i(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(6usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_i(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(6usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn rsv1(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(7usize, 1u8) as u8) }
    }

    #[inline]
    pub fn new_bitfield_1(
        pmport: cty::c_uchar,
        rsv0: cty::c_uchar,
        i: cty::c_uchar,
        rsv1: cty::c_uchar,
    ) -> internal::bitfield<[u8; 1usize], u8> {
        let mut bitfield: internal::bitfield<[u8; 1usize], u8> = Default::default();
        bitfield.set(0usize, 4u8, {
            let pmport: u8 = unsafe { transmute(pmport) };
            pmport as u64
        });
        bitfield.set(4usize, 2u8, {
            let rsv0: u8 = unsafe { transmute(rsv0) };
            rsv0 as u64
        });
        bitfield.set(6usize, 1u8, {
            let i: u8 = unsafe { transmute(i) };
            i as u64
        });
        bitfield.set(7usize, 1u8, {
            let rsv1: u8 = unsafe { transmute(rsv1) };
            rsv1 as u64
        });
        bitfield
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct FisData {
    pub fis_type: cty::c_uchar,
    _bitfield_1: internal::bitfield<[u8; 1usize], u8>,
    pub rsv1: [cty::c_uchar; 2usize],
    pub data: [cty::c_ulong; 1usize],
}

impl FisData {
    #[inline]
    pub fn pmport(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(0usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmport(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn rsv0(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(4usize, 4u8) as u8) }
    }

    #[inline]
    pub fn new_bitfield_1(
        pmport: cty::c_uchar,
        rsv0: cty::c_uchar,
    ) -> internal::bitfield<[u8; 1usize], u8> {
        let mut bitfield: internal::bitfield<[u8; 1usize], u8> = Default::default();
        bitfield.set(0usize, 4u8, {
            let pmport: u8 = unsafe { transmute(pmport) };
            pmport as u64
        });
        bitfield.set(4usize, 4u8, {
            let rsv0: u8 = unsafe { transmute(rsv0) };
            rsv0 as u64
        });
        bitfield
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct FisPioSetup {
    pub fis_type: cty::c_uchar,
    _bitfield_1: internal::bitfield<[u8; 1usize], u8>,
    pub status: cty::c_uchar,
    pub error: cty::c_uchar,
    pub lba0: cty::c_uchar,
    pub lba1: cty::c_uchar,
    pub lba2: cty::c_uchar,
    pub device: cty::c_uchar,
    pub lba3: cty::c_uchar,
    pub lba4: cty::c_uchar,
    pub lba5: cty::c_uchar,
    pub rsv2: cty::c_uchar,
    pub count_lo: cty::c_uchar,
    pub count_hi: cty::c_uchar,
    pub rsv3: cty::c_uchar,
    pub e_status: cty::c_uchar,
    pub tc: cty::c_ushort,
    pub rsv4: [cty::c_uchar; 2usize],
}

impl FisPioSetup {
    #[inline]
    pub fn pmport(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(0usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmport(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn d(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(5usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_d(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(5usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn i(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(6usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_i(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(6usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn new_bitfield_1(
        pmport: cty::c_uchar,
        rsv0: cty::c_uchar,
        d: cty::c_uchar,
        i: cty::c_uchar,
        rsv1: cty::c_uchar,
    ) -> internal::bitfield<[u8; 1usize], u8> {
        let mut bitfield: internal::bitfield<[u8; 1usize], u8> = Default::default();
        bitfield.set(0usize, 4u8, {
            let pmport: u8 = unsafe { transmute(pmport) };
            pmport as u64
        });
        bitfield.set(4usize, 1u8, {
            let rsv0: u8 = unsafe { transmute(rsv0) };
            rsv0 as u64
        });
        bitfield.set(5usize, 1u8, {
            let d: u8 = unsafe { transmute(d) };
            d as u64
        });
        bitfield.set(6usize, 1u8, {
            let i: u8 = unsafe { transmute(i) };
            i as u64
        });
        bitfield.set(7usize, 1u8, {
            let rsv1: u8 = unsafe { transmute(rsv1) };
            rsv1 as u64
        });
        bitfield
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct FisDmaSetup {
    pub fis_type: cty::c_uchar,
    _bitfield_1: internal::bitfield<[u8; 1usize], u8>,
    rsved: [cty::c_uchar; 2usize],
    pub dma_buf_id: cty::c_ushort,
    pub dma_buf_id2: cty::c_ushort,
    rsvd: cty::c_ulong,
    pub dma_buf_offset: cty::c_ulong,
    pub transfer_count: cty::c_ulong,
    resvd: cty::c_ulong,
}

impl FisDmaSetup {
    #[inline]
    pub fn pmport(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(0usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmport(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn d(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(5usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_d(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(5usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn i(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(6usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_i(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(6usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn a(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(7usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_a(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(7usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn new_bitfield_1(
        pmport: cty::c_uchar,
        rsv0: cty::c_uchar,
        d: cty::c_uchar,
        i: cty::c_uchar,
        a: cty::c_uchar,
    ) -> internal::bitfield<[u8; 1usize], u8> {
        let mut bitfield: internal::bitfield<[u8; 1usize], u8> = Default::default();
        bitfield.set(0usize, 4u8, {
            let pmport: u8 = unsafe { transmute(pmport) };
            pmport as u64
        });
        bitfield.set(4usize, 1u8, {
            let rsv0: u8 = unsafe { transmute(rsv0) };
            rsv0 as u64
        });
        bitfield.set(5usize, 1u8, {
            let d: u8 = unsafe { transmute(d) };
            d as u64
        });
        bitfield.set(6usize, 1u8, {
            let i: u8 = unsafe { transmute(i) };
            i as u64
        });
        bitfield.set(7usize, 1u8, {
            let a: u8 = unsafe { transmute(a) };
            a as u64
        });
        bitfield
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HbaFis {
    pub dsfis: FisDmaSetup,
    pad0: [cty::c_uchar; 4usize],
    pub psfis: FisPioSetup,
    pad1: [cty::c_uchar; 12usize],
    pub rfis: FisRegD2H,
    pad2: [cty::c_uchar; 4usize],
    pub sdbfis: cty::c_ushort,
    pub ufis: [cty::c_uchar; 64usize],
    rsv: [cty::c_uchar; 96usize],
}

impl Default for HbaFis {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

impl ::core::fmt::Debug for HbaFis {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(
            f,
            "HbaFis{{ dsfis: {:?}, psfis: {:?}, rfis: {:?}, sdbfis: {:?}, ufis: [...]}}",
            self.dsfis, self.psfis, self.rfis, self.sdbfis
        )
    }
}

impl ::core::cmp::PartialEq for HbaFis {
    fn eq(&self, other: &HbaFis) -> bool {
        self.dsfis == other.dsfis
            && self.psfis == other.psfis
            && self.rfis == other.rfis
            && self.sdbfis == other.sdbfis
            && &self.ufis[..] == &other.ufis[..]
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct HbaCmdHeader {
    _bitfield_1: internal::bitfield<[u8; 2usize], u8>,
    pub prdtl: cty::c_ushort,
    pub prdbc: cty::c_ulong,
    pub ctba: cty::c_ulong,
    pub ctbau: cty::c_ulong,
    rsv1: [cty::c_ulong; 4usize],
}

impl HbaCmdHeader {
    #[inline]
    pub fn cfl(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(0usize, 5u8) as u8) }
    }

    #[inline]
    pub fn set_cfl(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 5u8, val as u64)
        }
    }

    #[inline]
    pub fn a(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(5usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_a(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(5usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn w(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(6usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_w(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(6usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn p(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(7usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_p(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(7usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn r(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(8usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_r(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(8usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn b(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(9usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_b(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(9usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn c(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(10usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_c(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(10usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn pmp(&self) -> cty::c_uchar {
        unsafe { transmute(self._bitfield_1.get(12usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmp(&mut self, val: cty::c_uchar) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(12usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn new_bitfield_1(
        cfl: cty::c_uchar,
        a: cty::c_uchar,
        w: cty::c_uchar,
        p: cty::c_uchar,
        r: cty::c_uchar,
        b: cty::c_uchar,
        c: cty::c_uchar,
        rsv0: cty::c_uchar,
        pmp: cty::c_uchar,
    ) -> internal::bitfield<[u8; 2usize], u8> {
        let mut bitfield: internal::bitfield<[u8; 2usize], u8> = Default::default();
        bitfield.set(0usize, 5u8, {
            let cfl: u8 = unsafe { transmute(cfl) };
            cfl as u64
        });
        bitfield.set(5usize, 1u8, {
            let a: u8 = unsafe { transmute(a) };
            a as u64
        });
        bitfield.set(6usize, 1u8, {
            let w: u8 = unsafe { transmute(w) };
            w as u64
        });
        bitfield.set(7usize, 1u8, {
            let p: u8 = unsafe { transmute(p) };
            p as u64
        });
        bitfield.set(8usize, 1u8, {
            let r: u8 = unsafe { transmute(r) };
            r as u64
        });
        bitfield.set(9usize, 1u8, {
            let b: u8 = unsafe { transmute(b) };
            b as u64
        });
        bitfield.set(10usize, 1u8, {
            let c: u8 = unsafe { transmute(c) };
            c as u64
        });
        bitfield.set(11usize, 1u8, {
            let rsv0: u8 = unsafe { transmute(rsv0) };
            rsv0 as u64
        });
        bitfield.set(12usize, 4u8, {
            let pmp: u8 = unsafe { transmute(pmp) };
            pmp as u64
        });
        bitfield
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct HbaPrdtEntry {
    pub dba: cty::c_ulong,
    pub dbau: cty::c_ulong,
    rsv0: cty::c_ulong,
    _bitfield_1: internal::bitfield<[u8; 4usize], u32>,
}

impl HbaPrdtEntry {
    #[inline]
    pub fn dbc(&self) -> cty::c_ulong {
        unsafe { transmute(self._bitfield_1.get(0usize, 22u8) as cty::c_ulong) }
    }

    #[inline]
    pub fn set_dbc(&mut self, val: cty::c_ulong) {
        unsafe {
            let val: cty::c_ulong = transmute(val);
            self._bitfield_1.set(0usize, 22u8, val as u64)
        }
    }

    #[inline]
    pub fn i(&self) -> cty::c_ulong {
        unsafe { transmute(self._bitfield_1.get(31usize, 1u8) as cty::c_ulong) }
    }

    #[inline]
    pub fn set_i(&mut self, val: cty::c_ulong) {
        unsafe {
            let val: cty::c_ulong = transmute(val);
            self._bitfield_1.set(31usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn new_bitfield_1(
        dbc: cty::c_ulong,
        rsv1: cty::c_ulong,
        i: cty::c_ulong,
    ) -> internal::bitfield<[u8; 4usize], u32> {
        let mut bitfield: internal::bitfield<[u8; 4usize], u32> = Default::default();
        bitfield.set(0usize, 22u8, {
            let dbc: cty::c_ulong = unsafe { transmute(dbc) };
            dbc as u64
        });
        bitfield.set(22usize, 9u8, {
            let rsv1: cty::c_ulong = unsafe { transmute(rsv1) };
            rsv1 as u64
        });
        bitfield.set(31usize, 1u8, {
            let i: cty::c_ulong = unsafe { transmute(i) };
            i as u64
        });
        bitfield
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HbaCmdTbl {
    pub cfis: [cty::c_uchar; 64usize],
    pub acmd: [cty::c_uchar; 16usize],
    rsv: [cty::c_uchar; 48usize],
    pub prdt_entry: [HbaPrdtEntry; 65535usize],
}

impl Default for HbaCmdTbl {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

impl ::core::fmt::Debug for HbaCmdTbl {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(
            f,
            "HbaCmdTbl{{ cfis: [...], acmd: {:?}, prdt_entry: [...] }}",
            self.acmd
        )
    }
}

impl ::core::cmp::PartialEq for HbaCmdTbl {
    fn eq(&self, other: &HbaCmdTbl) -> bool {
        &self.cfis[..] == &other.cfis[..]
            && self.acmd == other.acmd
            && &self.prdt_entry[..] == &other.prdt_entry[..]
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct HbaPort {
    pub clb: cty::c_ulong,
    pub clbu: cty::c_ulong,
    pub fb: cty::c_ulong,
    pub fbu: cty::c_ulong,
    pub is: cty::c_ulong,
    pub ie: cty::c_ulong,
    pub cmd: cty::c_ulong,
    rsv0: cty::c_ulong,
    pub tfd: cty::c_ulong,
    pub sig: cty::c_ulong,
    pub ssts: cty::c_ulong,
    pub sctl: cty::c_ulong,
    pub serr: cty::c_ulong,
    pub sact: cty::c_ulong,
    pub ci: cty::c_ulong,
    pub sntf: cty::c_ulong,
    pub fbs: cty::c_ulong,
    rsv1: [cty::c_ulong; 11usize],
    pub vendor: [cty::c_ulong; 4usize],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HbaMem {
    pub cap: cty::c_ulong,
    pub ghc: cty::c_ulong,
    pub is: cty::c_ulong,
    pub pi: cty::c_ulong,
    pub vs: cty::c_ulong,
    pub ccc_ctl: cty::c_ulong,
    pub ccc_pts: cty::c_ulong,
    pub em_loc: cty::c_ulong,
    pub em_ctl: cty::c_ulong,
    pub cap2: cty::c_ulong,
    pub bohc: cty::c_ulong,
    rsv: [cty::c_uchar; 116],
    pub vendor: [cty::c_uchar; 96],
    pub ports: [HbaPort; 32],
}

impl Default for HbaMem {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

impl ::core::fmt::Debug for HbaMem {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "HbaMem {{cap: {:X}, ghc: {:X}, is: {:X}, pi: {:X}, vs: {:X}, ccc_ctl: {:X}, ccc_pts: {:X}, em_loc: {:X}, em_ctl: {:X}, cap2: {:X}, bohc: {:X}}}", self.cap, self.ghc, self.is, self.pi, self.vs, self.ccc_ctl, self.ccc_pts, self.em_loc, self.em_ctl, self.cap2, self.bohc)
    }
}

impl ::core::cmp::PartialEq for HbaMem {
    fn eq(&self, other: &HbaMem) -> bool {
        self.cap == other.cap
            && self.ghc == other.ghc
            && self.is == other.is
            && self.pi == other.pi
            && self.vs == other.vs
            && self.ccc_ctl == other.ccc_ctl
            && self.ccc_pts == other.ccc_pts
            && self.em_loc == other.em_loc
            && self.em_ctl == other.em_ctl
            && self.cap2 == other.cap2
            && self.bohc == other.bohc
            && &self.vendor[..] == &other.vendor[..]
            && &self.ports[..] == &other.ports[..]
    }
}

pub fn init() {
    for dev in pci::get_devices() {
        if dev.class == 0x01 && dev.subclass == 0x06 && dev.prog_if == 0x01 {
            printkln!(
                "AHCI: found AHCI-capable device with vendor {:X} and device {:X}",
                dev.vendor,
                dev.device
            );
            let mut hbadb = HBADB.lock();
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
                    printkln!("AHCI: skipping AHCI device {:X}:{:X}: AHCI device has 16-bit BAR address {:X}", dev.vendor, dev.device, bars[5]);
                    continue;
                }
                allocate_phys_range(bars[5], bars[5] + 0x28);
                printkln!("AHCI: detected base address for AHCI driver: {:X}", bars[5]);
                let mut pos = usize::max_value();
                for (i, hba) in hbadb.iter().enumerate() {
                    if hba.bar == 0 && hba.idx == 0 {
                        pos = i;
                        printkln!("AHCI: Inserting device at position {}", i);
                        break;
                    }
                }
                if pos != usize::max_value() {
                    hbadb[pos].bar = bars[5];
                    hbadb[pos].idx = pos;
                    hbadb[pos].device = dev;
                    let mem = unsafe { &mut *(hbadb[pos].bar as *mut HbaMem) };
                    printkln!("AHCI: Device scan: dbg: {:?}", mem);
                    let pi = mem.pi;
                    for i in 0..32 {
                        if pi.get_bit(i) {
                            let mut port = mem.ports[i];
                            let ssts = port.ssts;
                            let ipm = (ssts >> 8) & 0x0F;
                            let det = ipm & 0x0F;
                            if det != HBAPortStatus::DetPresent as u64
                                && ipm != HBAPortStatus::IpmActive as u64
                            {
                                printkln!("AHCI: warning: not initializing port {} because DET and IPM are not valid", i);
                            } else if port.sig == SIG_ATAPI {
                                printkln!("AHCI: Port {}: ATAPI device found, but ATAPI devices are not supported. Skipping", i);
                            } else if port.sig == SIG_SATA {
                                printkln!("AHCI: Port {}: SATA device found", i);
                                rebase_port(&mut port, i as u64);
                                let mut buffer: u16 = 0x1000;
                                ata_read(&mut port, 0, 0, 1, &mut buffer);
                                let mut data: Vec<u8> = Vec::new();
                                for j in 0..512 {
                                    data.push(read_memory((buffer as u64) + j) as u8);
                                }
                                if data[510] == 0x55 && data[511] == 0xAA {
                                    printkln!("AHCI: port {}: device is bootable", i);
                                }
                                let mut base = 0x01BE;
                                for i in 0..4 {
                                    printkln!(
                                        "AHCI: Port {}: MBR = {:X}, active = {:X}, FS = {:X}",
                                        i,
                                        i,
                                        data[base],
                                        data[base + 4]
                                    );
                                    base += 16;
                                }
                            }
                        }
                    }
                } else {
                    printkln!("AHCI: error: Cannot add HBA {:X}:{:X} to the internal HBA list: HBA maximum reached.", dev.vendor, dev.device);
                    continue;
                }
            }
        }
    }
}

pub fn start_command_engine(port: &mut HbaPort) {
    loop {
        if !port.cmd.get_bit(PortCommand::Cr as usize) {
            break;
        }
    }
    port.cmd |= PortCommand::Fre as u64;
    port.cmd |= PortCommand::St as u64;
}

pub fn stop_command_engine(port: &mut HbaPort) {
    port.cmd &= !(PortCommand::St as u64);
    loop {
        if port.cmd.get_bit(PortCommand::Fr as usize) {
            continue;
        }
        if port.cmd.get_bit(PortCommand::Cr as usize) {
            continue;
        }
        break;
    }
    port.cmd &= !(PortCommand::Fre as u64);
}

pub fn rebase_port(port: &mut HbaPort, new_port: u64) {
    stop_command_engine(port);
    port.clb = AHCI_BASE + (new_port << 10);
    port.clbu = 0;
    unsafe {
        write_bytes(port.clb as *mut u64, 0, 1024);
    }
    port.fb = AHCI_BASE + (32 << 10) + (new_port << 8);
    port.fbu = 0;
    unsafe {
        write_bytes(port.fb as *mut u64, 0, 256);
    }
    unsafe {
        let raw_header = port.clb as *mut HbaCmdHeader;
        for i in 0..32 {
            let header = raw_header.offset(i).as_mut().unwrap() as &mut HbaCmdHeader;
            header.prdtl = 8;
            header.ctba = AHCI_BASE + (40 << 10) + (new_port << 13) + (i << 8) as u64;
            header.ctbau = 0;
            write_bytes(header.ctba as *mut u64, 0, 256);
        }
    }
    start_command_engine(port);
}

pub fn find_cmd_slot(port: &mut HbaPort) -> i32 {
    let mut slots = port.sact | port.ci;
    for i in 0..32 {
        if (slots & 1) == 0 {
            return i;
        }
        slots >>= 1;
    }
    printkln!("AHCI: fatal: cannot find free command slot");
    return -1;
}

pub fn ata_read(
    port: &mut HbaPort,
    start_lo: u64,
    start_hi: u64,
    count: u64,
    buf: &mut u16,
) -> bool {
    port.is = 0;
    let mut cnt = count.clone();
    let mut spin = 0;
    let slot = find_cmd_slot(port);
    if slot == -1 {
        return false;
    }
    let header = unsafe {
        let raw_ptr = port.clb as *mut HbaCmdHeader;
        raw_ptr.offset(slot as isize).as_mut().unwrap() as &mut HbaCmdHeader
    };
    header.set_cfl((size_of::<FisRegH2D>() / size_of::<cty::c_ulong>()) as u8);
    header.set_w(0);
    header.prdtl = (((count - 1) >> 4) + 1) as u16;
    let cmdtbl = unsafe {
        let raw_ptr = header.ctba as *mut HbaCmdTbl;
        raw_ptr.as_mut().unwrap() as &mut HbaCmdTbl
    };
    unsafe {
        let size: u16 = (size_of::<HbaCmdTbl>() as u16)
            + ((header.prdtl - 1) as u16) * (size_of::<HbaPrdtEntry>() as u16);
        write_bytes(cmdtbl, 0, size as usize);
    }
    let mut i: usize = 0;
    while i < (header.prdtl as usize) - 1 {
        cmdtbl.prdt_entry[i].dba = *buf as cty::c_ulong;
        cmdtbl.prdt_entry[i].set_dbc(0x1FFF);
        cmdtbl.prdt_entry[i].set_i(1);
        *buf += 0x1000;
        cnt -= 16;
        i += 1;
    }
    cmdtbl.prdt_entry[i].dba = *buf as cty::c_ulong;
    cmdtbl.prdt_entry[i].set_dbc((cnt << 9) - 1);
    cmdtbl.prdt_entry[i].set_i(1);
    let ptr = &mut cmdtbl.cfis;
    let cmdfis = unsafe { &mut *(ptr as *mut [u8; 64] as *mut FisRegH2D) };
    cmdfis.fis_type = FisType::RegH2D as cty::c_uchar;
    cmdfis.set_c(1);
    cmdfis.command = AhciCommand::ReadDmaExt as cty::c_uchar;
    cmdfis.lba0 = start_lo as cty::c_uchar;
    cmdfis.lba1 = (start_lo >> 8) as cty::c_uchar;
    cmdfis.lba2 = (start_lo >> 16) as cty::c_uchar;
    cmdfis.device = 0x40;
    cmdfis.lba3 = (start_lo >> 24) as cty::c_uchar;
    cmdfis.lba4 = start_hi as cty::c_uchar;
    cmdfis.lba5 = (start_hi >> 8) as cty::c_uchar;
    cmdfis.count_lo = (cnt as u8) & 0xFF;
    cmdfis.count_hi = ((cnt as u8) >> 8) & 0xFF;
    while (port.tfd & (AtaStatus::Busy as u64 | AtaStatus::Drq as u64) > 0) && spin < 1000000 {
        spin += 1;
    }
    if spin == 1000000 {
        panic!("Detected port hang: {:?}", port);
    }
    port.ci = 1 << slot;
    loop {
        if port.ci & (1 << slot) == 0 {
            break;
        }
        if port.is.get_bit(30) {
            panic!("Read error with HBA port: {:?}", port);
        }
    }
    if port.is.get_bit(30) {
        panic!("Read error with HBA port: {:?}", port);
    }
    return true;
}
