const GPT_SIG: u64 = 0x5452415020494645;
#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub struct GPTPartition {
pub partition_type_guid: u128,
pub partition_guid: u128,
pub start_lba: u64,
pub end_lba: u64,
pub attrib_flags: u64,
pub part_name: [u8; 72],
}

