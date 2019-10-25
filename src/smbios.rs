use crate::memory::*;

const SMBIOS_SIG: &'static [u8; 5] = b"_SM3_";

pub fn init() {
allocate_phys_range(0x000F0000, 0x000FFFFF-0x000F0000);
printkln!("SMBIOS: Searching for SMBios area");
for i: *mut u8 in 0x000F0000 as *mut u8 ..= 0x000FFFFF as *mut u8{
unsafe {
if i.read_volatile() 