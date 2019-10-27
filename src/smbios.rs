use crate::memory::*;
use crate::printkln;
use core::slice;
use core::mem::size_of;

const SMBIOS_SIG: &'static [u8; 5] = b"_SM3_";

#[repr(C)]
#[derive(Debug)]
pub struct SMBIOSTable {
pub anchor: [u8; 5],
pub entry_checksum: u8,
pub entry_len: u8,
pub major: u8,
pub minor: u8,
pub doc_rev: u8,
pub entry_rev: u8,
rsv: u8,
pub struct_tbl_max_sz: u32,
pub struct_tbl_addr: u64,
}

pub fn init() {
allocate_phys_range(0x000F0000, 0x000FFFFF);
let mut length: u64 = 0;
let mut i: u64 = 0;
let mut checksum: u16 = u16::max_value();
let mut entry_tbl: &mut SMBIOSTable;
printkln!("SMBIOS: Searching for SMBios area");
let mut j: u32 = 0x000F0000;
while j < 0x000FFFFF {
let addr = j as *mut u16;
entry_tbl = unsafe { &mut *(addr as *mut SMBIOSTable) };
if entry_tbl.anchor == *SMBIOS_SIG {
length = entry_tbl.entry_len as u64;
checksum = 0;
for k in 0 .. length {
checksum += unsafe { addr.offset(k as isize).read_volatile() } as u16;
}
}
if checksum == 0 {
i = j as u64;
}
j += 16;
}
if i == 0x000FFFFF {
printkln!("SMBIOS: no SMBIOS area found");
return;
}
printkln!("SMBIOS: found SMBIOS area at addr {:X}h", i);
let entry_tbl = unsafe { &mut *(i as *mut SMBIOSTable) };
printkln!("SMBIOS: Smbios table: {:?}", entry_tbl);
}
