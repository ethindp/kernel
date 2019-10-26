use core::mem::{transmute, zeroed};
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct bitfield<Storage, Align>
where
    Storage: AsRef<[u8]> + AsMut<[u8]>,
{
    storage: Storage,
    align: [Align; 0],
}

impl<Storage, Align> bitfield<Storage, Align>
where
    Storage: AsRef<[u8]> + AsMut<[u8]>,
{
    #[inline]
    pub fn new(storage: Storage) -> Self {
        Self { storage, align: [] }
    }

    #[inline]
    pub fn get_bit(&self, index: usize) -> bool {
        assert!(index / 8 < self.storage.as_ref().len());
        let byte_index = index / 8;
        let byte = self.storage.as_ref()[byte_index];
        let bit_index = if cfg!(target_endian = "big") {
            7 - (index % 8)
        } else {
            index % 8
        };
        let mask = 1 << bit_index;
        byte & mask == mask
    }

    #[inline]
    pub fn set_bit(&mut self, index: usize, val: bool) {
        assert!(index / 8 < self.storage.as_ref().len());
        let byte_index = index / 8;
        let byte = &mut self.storage.as_mut()[byte_index];
        let bit_index = if cfg!(target_endian = "big") {
            7 - (index % 8)
        } else {
            index % 8
        };
        let mask = 1 << bit_index;
        if val {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
    }

    #[inline]
    pub fn get(&self, bit_offset: usize, bit_width: u8) -> u64 {
        assert!(bit_width <= 64);
        assert!(bit_offset / 8 < self.storage.as_ref().len());
        assert!((bit_offset + (bit_width as usize)) / 8 <= self.storage.as_ref().len());
        let mut val = 0;
        for i in 0..(bit_width as usize) {
            if self.get_bit(i + bit_offset) {
                let index = if cfg!(target_endian = "big") {
                    bit_width as usize - 1 - i
                } else {
                    i
                };
                val |= 1 << index;
            }
        }
        val
    }

    #[inline]
    pub fn set(&mut self, bit_offset: usize, bit_width: u8, val: u64) {
        assert!(bit_width <= 64);
        assert!(bit_offset / 8 < self.storage.as_ref().len());
        assert!((bit_offset + (bit_width as usize)) / 8 <= self.storage.as_ref().len());
        for i in 0..(bit_width as usize) {
            let mask = 1 << i;
            let val_bit_is_set = val & mask == mask;
            let index = if cfg!(target_endian = "big") {
                bit_width as usize - 1 - i
            } else {
                i
            };
            self.set_bit(index + bit_offset, val_bit_is_set);
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct FisRegH2D {
    pub fis_type: u8,
    _bitfield_1: bitfield<[u8; 1usize], u8>,
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
    rsv1: [u8; 4usize],
}

impl FisRegH2D {
    #[inline]
    pub fn pmport(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(0usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmport(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn c(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(7usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_c(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(7usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn new_bitfield_1(pmport: u8, rsv0: u8, c: u8) -> bitfield<[u8; 1usize], u8> {
        let mut bitfield: bitfield<[u8; 1usize], u8> = Default::default();
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
    pub fis_type: u8,
    _bitfield_1: bitfield<[u8; 1usize], u8>,
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
    pub count_lo: u8,
    pub count_hi: u8,
    pub rsv3: [u8; 2usize],
    pub rsv4: [u8; 4usize],
}

impl FisRegD2H {
    #[inline]
    pub fn pmport(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(0usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmport(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn rsv0(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(4usize, 2u8) as u8) }
    }

    #[inline]
    pub fn i(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(6usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_i(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(6usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn rsv1(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(7usize, 1u8) as u8) }
    }

    #[inline]
    pub fn new_bitfield_1(pmport: u8, rsv0: u8, i: u8, rsv1: u8) -> bitfield<[u8; 1usize], u8> {
        let mut bitfield: bitfield<[u8; 1usize], u8> = Default::default();
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
    pub fis_type: u8,
    _bitfield_1: bitfield<[u8; 1usize], u8>,
    pub rsv1: [u8; 2usize],
    pub data: [u32; 1usize],
}

impl FisData {
    #[inline]
    pub fn pmport(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(0usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmport(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn rsv0(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(4usize, 4u8) as u8) }
    }

    #[inline]
    pub fn new_bitfield_1(pmport: u8, rsv0: u8) -> bitfield<[u8; 1usize], u8> {
        let mut bitfield: bitfield<[u8; 1usize], u8> = Default::default();
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
    pub fis_type: u8,
    _bitfield_1: bitfield<[u8; 1usize], u8>,
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
    pub count_lo: u8,
    pub count_hi: u8,
    pub rsv3: u8,
    pub e_status: u8,
    pub tc: u16,
    pub rsv4: [u8; 2usize],
}

impl FisPioSetup {
    #[inline]
    pub fn pmport(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(0usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmport(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn d(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(5usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_d(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(5usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn i(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(6usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_i(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(6usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn new_bitfield_1(
        pmport: u8,
        rsv0: u8,
        d: u8,
        i: u8,
        rsv1: u8,
    ) -> bitfield<[u8; 1usize], u8> {
        let mut bitfield: bitfield<[u8; 1usize], u8> = Default::default();
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
    pub fis_type: u8,
    _bitfield_1: bitfield<[u8; 1usize], u8>,
    rsved: [u8; 2usize],
    pub dma_buf_id: u64,
    rsvd: u32,
    pub dma_buf_offset: u32,
    pub transfer_count: u32,
    resvd: u32,
}

impl FisDmaSetup {
    #[inline]
    pub fn pmport(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(0usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmport(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn d(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(5usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_d(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(5usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn i(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(6usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_i(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(6usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn a(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(7usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_a(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(7usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn new_bitfield_1(pmport: u8, rsv0: u8, d: u8, i: u8, a: u8) -> bitfield<[u8; 1usize], u8> {
        let mut bitfield: bitfield<[u8; 1usize], u8> = Default::default();
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
    pad0: [u8; 4usize],
    pub psfis: FisPioSetup,
    pad1: [u8; 12usize],
    pub rfis: FisRegD2H,
    pad2: [u8; 4usize],
    pub sdbfis: u16,
    pub ufis: [u8; 64usize],
    rsv: [u8; 96usize],
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
    _bitfield_1: bitfield<[u8; 2usize], u8>,
    pub prdtl: u16,
    pub prdbc: u32,
    pub ctba: u32,
    pub ctbau: u32,
    rsv1: [u32; 4usize],
}

impl HbaCmdHeader {
    #[inline]
    pub fn cfl(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(0usize, 5u8) as u8) }
    }

    #[inline]
    pub fn set_cfl(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(0usize, 5u8, val as u64)
        }
    }

    #[inline]
    pub fn a(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(5usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_a(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(5usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn w(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(6usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_w(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(6usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn p(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(7usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_p(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(7usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn r(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(8usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_r(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(8usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn b(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(9usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_b(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(9usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn c(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(10usize, 1u8) as u8) }
    }

    #[inline]
    pub fn set_c(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(10usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn pmp(&self) -> u8 {
        unsafe { transmute(self._bitfield_1.get(12usize, 4u8) as u8) }
    }

    #[inline]
    pub fn set_pmp(&mut self, val: u8) {
        unsafe {
            let val: u8 = transmute(val);
            self._bitfield_1.set(12usize, 4u8, val as u64)
        }
    }

    #[inline]
    pub fn new_bitfield_1(
        cfl: u8,
        a: u8,
        w: u8,
        p: u8,
        r: u8,
        b: u8,
        c: u8,
        rsv0: u8,
        pmp: u8,
    ) -> bitfield<[u8; 2usize], u8> {
        let mut bitfield: bitfield<[u8; 2usize], u8> = Default::default();
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
    pub dba: u32,
    pub dbau: u32,
    rsv0: u32,
    _bitfield_1: bitfield<[u8; 4usize], u32>,
}

impl HbaPrdtEntry {
    #[inline]
    pub fn dbc(&self) -> u32 {
        unsafe { transmute(self._bitfield_1.get(0usize, 22u8) as u32) }
    }

    #[inline]
    pub fn set_dbc(&mut self, val: u32) {
        unsafe {
            let val: u32 = transmute(val);
            self._bitfield_1.set(0usize, 22u8, val as u64)
        }
    }

    #[inline]
    pub fn i(&self) -> u32 {
        unsafe { transmute(self._bitfield_1.get(31usize, 1u8) as u32) }
    }

    #[inline]
    pub fn set_i(&mut self, val: u32) {
        unsafe {
            let val: u32 = transmute(val);
            self._bitfield_1.set(31usize, 1u8, val as u64)
        }
    }

    #[inline]
    pub fn new_bitfield_1(dbc: u32, rsv1: u32, i: u32) -> bitfield<[u8; 4usize], u32> {
        let mut bitfield: bitfield<[u8; 4usize], u32> = Default::default();
        bitfield.set(0usize, 22u8, {
            let dbc: u32 = unsafe { transmute(dbc) };
            dbc as u64
        });
        bitfield.set(22usize, 9u8, {
            let rsv1: u32 = unsafe { transmute(rsv1) };
            rsv1 as u64
        });
        bitfield.set(31usize, 1u8, {
            let i: u32 = unsafe { transmute(i) };
            i as u64
        });
        bitfield
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HbaCmdTbl {
    pub cfis: [u8; 64usize],
    pub acmd: [u8; 16usize],
    rsv: [u8; 48usize],
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
    rsv1: [u32; 11usize],
    pub vendor: [u32; 4usize],
}

#[repr(C)]
#[derive(Clone, Copy)]
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
