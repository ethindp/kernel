use super::ata;
use alloc::string::String;
use alloc::collections::linked_list::LinkedList;
use lazy_static::lazy_static;
use alloc::vec::Vec;

const GPT_SIG: u64 = 0x5452415020494645;

lazy_static! {
static ref CRC32_TBL: [u32; 256] = {
let mut crc32_table = [0; 256];
for n in 0..256 {
crc32_table[n as usize] = (0..8).fold(n as u32, |acc, _| {
match acc & 1 {
1 => 0xedb88320 ^ (acc >> 1),
_ => acc >> 1,
}
});
}
crc32_table
};
}

#[derive(Clone, Copy, Debug)]
pub struct GPTHeader {
pub sig: u64,
pub rev: u32,
pub header_size: u32,
pub crc: u32,
pub current_lba: u64,
pub backup_lba: u64,
pub primary_part_tbl_lba: u64,
pub secondary_part_tbl_lba: u64,
pub disk_guid: u128,
pub part_entry_begin: u64,
pub num_partitions: u32,
pub part_size: u32,
pub part_crc: u32,
}

#[derive(Clone, Debug)]
pub struct GPTPartition {
pub partition_type_guid: u128,
pub partition_guid: u128,
pub start_lba: u64,
pub end_lba: u64,
pub attrib_flags: u64,
pub part_name: String,
}

#[derive(Clone, Debug)]
pub struct GPTPartitionTable {
pub header: GPTHeader,
pub partitions: LinkedList<GPTPartition>,
}

pub fn read_gpt_header()->Option<GPTHeader> {
let lba = unsafe { ata::read_sectors_ext(1, 1, 1) };
if u64::from_le_bytes([lba[0], lba[1], lba[2], lba[3], lba[4], lba[5], lba[6], lba[7]]) != GPT_SIG {
return None;
}
Some(GPTHeader {
sig: u64::from_le_bytes([lba[0], lba[1], lba[2], lba[3], lba[4], lba[5], lba[6], lba[7]]),
rev: u32::from_le_bytes([lba[8], lba[9], lba[10], lba[11]]),
header_size: u32::from_le_bytes([lba[12], lba[13], lba[14], lba[15]]),
crc: u32::from_le_bytes([lba[16], lba[17], lba[18], lba[19]]),
current_lba: u64::from_le_bytes([lba[24], lba[25], lba[26], lba[27], lba[28], lba[29], lba[30], lba[31]]),
backup_lba: u64::from_le_bytes([lba[32], lba[33], lba[34], lba[35], lba[36], lba[37], lba[38], lba[39]]),
primary_part_tbl_lba: u64::from_le_bytes([lba[40], lba[41], lba[42], lba[43], lba[44], lba[45], lba[46], lba[47]]),
secondary_part_tbl_lba: u64::from_le_bytes([lba[48], lba[49], lba[50], lba[51], lba[52], lba[53], lba[54], lba[55]]),
disk_guid: u128::from_le_bytes([lba[56], lba[57], lba[58], lba[59], lba[60], lba[61], lba[62], lba[63], lba[64], lba[65], lba[66], lba[67], lba[68], lba[69], lba[70], lba[71]]),
part_entry_begin: u64::from_le_bytes([lba[72], lba[73], lba[74], lba[75], lba[76], lba[77], lba[78], lba[79]]),
num_partitions: u32::from_le_bytes([lba[80], lba[81], lba[82], lba[83]]),
part_size: u32::from_le_bytes([lba[84], lba[85], lba[86], lba[87]]),
part_crc: u32::from_le_bytes([lba[88], lba[89], lba[90], lba[91]]),
//Bytes 92-511 are reserved
})
}

pub fn read_gpt_partition_table()->Option<GPTPartitionTable> {
if let Some(header) = read_gpt_header() {
let mut table = GPTPartitionTable {
header,
partitions: LinkedList::new(),
};
for address in header.part_entry_begin .. header.primary_part_tbl_lba {
let lba = unsafe { ata::read_sectors_ext(1, address, 1) };
for partition in lba.chunks(header.part_size as usize) {
if u128::from_le_bytes([partition[0], partition[1], partition[2], partition[3], partition[4], partition[5], partition[6], partition[7], partition[8], partition[9], partition[10], partition[11], partition[12], partition[13], partition[14], partition[15]]) == 0 {
continue; // partition is unused (might not even be a partition)
}
// Assemble string for partition name
let part_name = {
let mut words: Vec<u16> = Vec::new();
for i in (56 .. 128).step_by(2) {
words.push(u16::from_le_bytes([partition[i], partition[i + 1]]));
}
match  String::from_utf16(words.as_slice()) {
Ok(name) => name,
Err(_) => return None,
}
};
table.partitions.push_back(GPTPartition {
partition_type_guid: u128::from_le_bytes([partition[0], partition[1], partition[2], partition[3], partition[4], partition[5], partition[6], partition[7], partition[8], partition[9], partition[10], partition[11], partition[12], partition[13], partition[14], partition[15]]),
partition_guid: u128::from_le_bytes([partition[16], partition[17], partition[18], partition[19], partition[20], partition[21], partition[22], partition[23], partition[24], partition[25], partition[26], partition[27], partition[28], partition[29], partition[30], partition[31]]),
start_lba: u64::from_le_bytes([partition[32], partition[33], partition[34], partition[35], partition[36], partition[37], partition[38], partition[39]]),
end_lba: u64::from_le_bytes([partition[40], partition[41], partition[42], partition[43], partition[44], partition[45], partition[46], partition[47]]),
attrib_flags: u64::from_le_bytes([partition[48], partition[49], partition[50], partition[51], partition[52], partition[53], partition[54], partition[55]]),
part_name,
});
}
}
Some(table)
} else {
None
}
}

