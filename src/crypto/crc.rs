use lazy_static::lazy_static;

const CRC_POLY_16: u16 = 0xA001;
const CRC_POLY_32: u32 = 0xEDB88320;
const CRC_POLY_CCITT: u16 = 0x1021;
const CRC_POLY_DNP: u16 = 0xA6BC;
const CRC_POLY_KERMIT: u16 = 0x8408;
const CRC_POLY_SICK: u16 = 0x8005;
const CRC64_ECMA182_POLY: u64 = 0x42F0E1EBA9EA3693;
const CRC_START_8: u8 = 0x00;
const CRC_START_16: u16 = 0x0000;
const CRC_START_MODBUS: u16 = 0xFFFF;
const CRC_START_XMODEM: u16 = 0x0000;
const CRC_START_CCITT_1D0F: u16 = 0x1D0F;
const CRC_START_CCITT_FFFF: u16 = 0xFFFF;
const CRC_START_KERMIT: u16 = 0x0000;
const CRC_START_SICK: u16 = 0x0000;
const CRC_START_DNP: u16 = 0x0000;
const CRC_START_32: u32 = 0xFFFFFFFF;

lazy_static! {
static ref CRC_TAB16: [u16; 256] = {
let mut table = [0u16; 256];
let mut crc: u16;
let mut c: u16;
for i in 0 .. 256 {
crc = 0;
c = i;
for _ in 0 .. 8 {
crc = if ((crc ^ c) & 0x0001) > 0 {
(crc >> 1) ^ CRC_POLY_16
} else {
crc >> 1
};
c = c >> 1;
}
table[i] = crc;
}
table
};
static ref CRC_TAB32: [0u32; 256] = {
let mut crc: u32;
let mut table = [0u32; 256];
for i in 0 .. 256 {
crc = i;
for _ in 0 .. 8 {
crc = if (crc & 0x00000001) > 0 {
(crc >> 1) ^ CRC_POLY_32
} else {
crc >> 1
};
}
table[i] = crc;
}
table
};
static ref CRC_TABCCITT: [u16; 256] = {
let mut table = [0u16; 256];
let mut crc: u16;
let mut c: u16;
for i in 0 .. 256 {
crc = 0;
c = i << 8;
for _ in 0 .. 8 {
crc = if ((crc ^ c) & 0x8000) > 0 {
(crc << 1) ^ CRC_POLY_CCITT
} else {
crc << 1
};
c = c << 1;
}
table[i] = crc;
}
table
};
static ref CRC_TABDNP: [u16; 256] = {
let mut table = [0u16; 256];
let mut crc: u16;
let mut c: u16;
for i in 0 .. 256 {
crc = 0;
c   = i as u16;
for _ in 0 .. 8 {
crc = if ((crc ^ c) & 0x0001) > 0 {
(crc >> 1) ^ CRC_POLY_DNP
} else {
crc >> 1
};
c = c >> 1;
}
table[i] = crc;
}
table
};
static ref CRC_TABKRMIT: [u16; 256] = {
let mut table = [0u16; 256];
let mut crc: u16;
let mut c: u16;
for i in 0 .. 256 {
crc = 0;
c = i;
for _ in 0 .. 8 {
crc = if (crc ^ c) & 0x0001) > 0 {
(crc >> 1) ^ CRC_POLY_KERMIT
} else {
crc >> 1
};
c = c >> 1;
}
table[i] = crc;
}
table
};
static ref CRC64_TAB: [u64; 65536] = {
let mut table = [0u64; 65536];
let mut crc: u64;
for i in 0 .. 65536 {
crc = i << 48;
for _ in 0 .. 16 {
crc = (crc << 1) ^ ((0 - (crc >> 63)) & CRC64_ECMA182_POLY);
}
table[((i & 0xff00) >> 8) | ((i & 0x00ff) << 8)] = ((crc & 0xff00ff00ff00ff00) >> 8) | ((crc & 0x00ff00ff00ff00ff) << 8);
}
table
};
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Mode {
CRC16,
Ccitt1d0f,
CcittFfff,
Dnp,
Kermit,
Sick,
Xmodem,
Modbus
}

/// Checksum8 represents an 8-bit checksum
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Checksum8(u8);

/// Checksum16 represents a 16-bit checksum
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Checksum16(u16);

/// Checksum32 represents a 32-bit checksum.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Checksum32(u32);

/// Checksum64 represents a 64-bit checksum
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Checksum64(u64);

impl Checksum16 {
pub fn compute(mode: Mode, input: &[u8]) -> self {
let mut crc: u16;
match mode {
Mode::CRC16 => {
let mut tmp: u16;
let mut short_c: u16;
crc = CRC_START_16;
for i in input.iter() {
short_c = 0x00ff & (i as u16);
tmp     =  crc       ^ short_c;
crc     = (crc >> 8) ^ CRC_TAB16[tmp & 0xff];
}
},
Mode::Ccitt1d0f => {
let mut tmp: u16;
let mut short_c: u16;
crc = CRC_START_CCITT_1D0F;
for i in input.iter() {
short_c = 0x00ff & (i as u16);
tmp     = (crc >> 8) ^ short_c;
crc     = (crc << 8) ^ CRC_TABCCITT[tmp];
}
},
Mode::CcittFfff => {
let mut tmp: u16;
let mut short_c: u16;
crc = CRC_START_CCITT_FFFF;
for i in input.iter() {
short_c = 0x00ff & (i as u16);
tmp     = (crc >> 8) ^ short_c;
crc     = (crc << 8) ^ CRC_TABCCITT[tmp];
}
},
Mode::Xmodem => {
let mut tmp: u16;
let mut short_c: u16;
crc = CRC_START_XMODEM;
for i in input.iter() {
short_c = 0x00ff & (i as u16);
tmp     = (crc >> 8) ^ short_c;
crc     = (crc << 8) ^ CRC_TABCCITT[tmp];
}
},
Mode::Dnp => {
let mut tmp: u16;
let mut short_c: u16;
let mut lo_byte: u16;
let mut hi_byte: u16;
crc = CRC_START_DNP;
for i in input.iter() {
short_c = 0x00ff & (i as u16);
tmp     =  crc       ^ short_c;
crc     = (crc >> 8) ^ CRC_TABDNP[tmp & 0xff];
}
crc       = !crc;
lo_byte  = (crc & 0xff00) >> 8;
hi_byte = (crc & 0x00ff) << 8;
crc       = lo_byte | hi_byte;
},
Mode::Kermit => {
let mut crc: u16;
let mut short_c: u16;
let mut lo_byte: u16;
let mut hi_byte: u16;
crc = CRC_START_KERMIT;
for i in input.iter() {
short_c = 0x00ff & (i as u16);
tmp     =  crc       ^ short_c;
crc     = (crc >> 8) ^ CRC_TABKRMIT[tmp & 0xff];
}
lo_byte = (crc & 0xff00) >> 8;
hi_byte = (crc & 0x00ff) << 8;
crc       = lo_byte | hi_byte;
},
Mode::Sick => {
let mut lo_byte: u16;
let mut hi_byte: u16;
let mut short_c: u16;
let mut short_p: u16;
crc     = CRC_START_SICK;
short_p = 0;
for i in input.iter() {
short_c = 0x00ff & (i as u16);
crc = if ( crc & 0x8000 ) > 0 {
( crc << 1 ) ^ CRC_POLY_SICK
} else {
crc << 1
};
crc    = crc ^ (short_c | short_p);
short_p = short_c << 8;
}
lo_byte = (crc & 0xff00) >> 8;
hi_byte = (crc & 0x00ff) << 8;
crc = lo_byte | hi_byte;
},
Mode::Modbus => {
let mut tmp: u16;
let mut short_c: u16;
crc = CRC_START_MODBUS;
for i in input.iter() {
short_c = 0x00ff & (i as u16);
tmp     =  crc       ^ short_c;
crc     = (crc >> 8) ^ CRC_TAB16[tmp & 0xff];
}
}
}
Checksum16(crc)
}

pub fn update(&mut self, mode: Mode, chr: u8, prev_chr: Option<u8>) {
match mode {
Mode::CRC16 => {
let short_c = 0x00ff & (chr as u16);
let tmp =  self.0       ^ short_c;
self.0 = (self.0 >> 8) ^ CRC_TAB16[tmp & 0xff];
},
Mode::Ccitt1d0f | Mode::CitFfff => {
let short_c  = 0x00ff & (chr as u16);
let tmp = (self.0 >> 8) ^ short_c;
self.0 = (self.0 << 8) ^ CRC_TABCCITT[tmp];
},
Mode::Dnp => {
let short_c = 0x00ff & (chr as u16);
let tmp = self.0 ^ short_c;
self.0 = (self.0 >> 8) ^ CRC_TABDNP[tmp & 0xff];
},
Mode::Kermit => {
let short_c = 0x00ff & (chr as u16);
let tmp = self.0 ^ short_c;
self.0 = (self.0 >> 8) ^ CRC_TABKRMIT[tmp & 0xff];
},
Mode::Sick => {
let short_c  =   0x00ff & (chr as u16);
let short_p  = ( 0x00ff & (prev_chr.unwrap_or(0) as u16)) << 8;
self.0 = if ( self.0 & 0x8000 ) > 0 {
( self.0 << 1 ) ^ CRC_POLY_SICK
} else {
self.0 = self.0 << 1
};
self.0 = self.0 & 0xffff;
self.0 = self.0 ^ ( short_c | short_p );
},
Mode::Xmodem => {
let short_c = 0x00ff & (chr as u16);
let tmp     = (self.0 >> 8) ^ short_c;
self.0     = (self.0 << 8) ^ CRC_TABCCITT[tmp];
},
Mode::Modbus => {
let short_c = 0x00ff & (chr as u16);
let tmp     =  self.0 ^ short_c;
self.0 = (self.0 >> 8) ^ CRC_TAB16[tmp & 0xff];
lo_byte = (self.0 & 0xff00) >> 8;
hi_byte = (self.0 & 0x00ff) << 8;
self.0 = lo_byte | hi_byte;
}
}
}
}


impl for Checksum32 {
pub fn compute(input: &[u8]) -> self {
let mut crc = CRC_START_32;
let mut long_c = 0u32;
let mut tmp = 0u32;
for i in input.iter() {
long_c = 0x000000FF & (i as u32);
tmp = crc ^ long_c;
crc = (crc >> 8) ^ CRC_TAB32[tmp & 0xff];
}
crc ^= 0xffffffff;
Checksum32(crc & 0xffffffff)
}

pub fn update(&mut self, chr: u8) {
let long_c = 0x000000ff& (chr as u32);
let tmp = self.0 ^ long_c;
self.0 = (self.0 >> 8) ^ CRC_TAB32[tmp & 0xff];
}
}

impl for Checksum64 {
// The below two methods need validation
pub fn compute(input: &[u8]) -> self {
let mut crc = 0u64;
for i in input.iter() {
crc = CRC64_TAB[(crc >> 48) ^ (i as u64)] ^ (crc << 16);
}
Checksum64(((crc & 0xff00ff00ff00ff00) >> 8) | ((crc & 0x00ff00ff00ff00ff) << 8))
}

pub fn update(&mut self, chr: u8) {
self.0 = CRC64_TAB[(self.0 >> 48) ^ (chr as u64)] ^ (self.0 << 16);
self.0 = ((self.0 & 0xff00ff00ff00ff00) >> 8) | ((self.0 & 0x00ff00ff00ff00ff) << 8);
}
}

