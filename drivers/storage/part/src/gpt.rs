use core::char::from_u32;
use heapless::{FnvIndexSet, String, Vec};
use heapless::consts::*;
use crc::crc32::checksum_ieee;
use zerocopy::FromBytes;
use itertools::Itertools;
use anyhow::{Result, anyhow};
use static_assertions::assert_eq_size;
use uuid;

const GPT_SIG: u64 = 0x5452_4150_2049_4645;

/// The partition table header defines the usable blocks on the disk. It also defines the number and size of the partition entries that make up the partition table.
/// See section 5.3.2 of the UEFI specification, v. 2.8, for extra
/// information on the GPT header and how software should handle it.
#[derive(Clone, Copy, Debug, Default, FromBytes)]
#[repr(C)]
pub struct Header {
/// Identifies EFI-compatible partition table header.
///
/// This value must contain the ASCII string "EFI PART", encoded as the 64-bit constant
/// 0x5452415020494645.
    pub sig: u64,
    /// The revision number for this header. This revision value is not related to the UEFI Specification
    /// version. This header is version 1.0, so the correct value is 0x00010000.
    pub revision: u32,
    /// Size in bytes of the GPT Header. The HeaderSize must be greater than or equal to 92 and must be less than or equal to the logical block size.
    pub header_size: u32,
    /// CRC32 checksum for the GPT Header structure.
    /// This value is computed by setting this field to 0, and computing the 32-bit CRC for HeaderSize bytes.
    pub crc32: u32,
    rsvd: u32,
    /// The LBA that contains this data structure.
    pub my_lba: u64,
    /// LBA address of the alternate GPT Header.
    pub alternate_lba: u64,
    /// The first usable logical block that may be used by a partition described by a GUID Partition Entry.
    pub first_usable_lba: u64,
    /// The last usable logical block that may be used by a partition described by a GUID Partition Entry.
    pub last_usable_lba: u64,
    /// GUID that can be used to uniquely identify the disk.
    pub disk_guid: u128,
    /// The starting LBA of the GUID Partition Entry array.
    pub part_entry_lba: u64,
    /// The number of Partition Entries in the GUID Partition Entry array.
    pub num_partitions: u32,
    /// The size, in bytes, of each of the GUID Partition Entry structures in the GUID Partition Entry array.
    /// This field shall be set to a value of 128 x 2 n where n is an integer greater than or equal to zero
    /// (e.g., 128, 256, 512, etc.).
    pub part_size: u32,
    /// The CRC32 of the GUID Partition Entry array. Starts at PartitionEntryLBA and is computed over a byte length of NumberOfPartitionEntries * SizeOfPartitionEntry.
    pub partition_entry_array_crc: u32,
}
assert_eq_size!(Header, [u8; 92]);

/// After the header, the Partition Entry Array describes partitions, using a minimum size of 128 bytes for each entry block.
///
/// The starting location of the array on disk, and the size of each entry, are given in the GPT header. The first 16 bytes of each entry designate the partition type's globally unique identifier (GUID). For example, the GUID for an
/// EFI system partition is C12A7328-F81F-11D2-BA4B-00A0C93EC93B. The second 16 bytes are a GUID unique to the partition. Then follow the starting and ending 64 bit LBAs, partition attributes, and the 36 character (max.)
/// Unicode partition name. As is the nature and purpose of GUIDs and as per RFC4122, no central registry is needed to ensure the uniqueness of the GUID partition type designators.
///
/// The 64-bit partition table attributes are shared between 48-bit common attributes for all partition types, and 16-bit type-specific attributes:
///
/// * Bit 0: Platform required (required by the computer to function properly, OEM partition for example, disk partitioning utilities must preserve the partition as is)
/// * Bit 1: EFI firmware should ignore the content of the partition and not try to read from it
/// * Bit 2: Legacy BIOS bootable (equivalent to active flag (typically bit 7 set) at offset +0h in partition entries of the MBR partition table)
/// * Bits 47:3: Reserved for future use
/// * Bits 63:48: Defined and used by the individual partition type
#[derive(Clone, Copy, Debug, Default, FromBytes)]
#[repr(C)]
pub struct Partition {
/// Unique ID that defines the purpose and type of this Partition. A value of zero defines that this partition entry is not being used.
    pub partition_type_guid: u128,
    /// GUID that is unique for every partition entry. Every partition ever created will have
    /// a unique GUID. This GUID must be assigned when the GPT Partition Entry is created.
    /// The GPT Partition Entry is created whenever the NumberOfPartitionEntries in the GPT
    /// Header is increased to include a larger range of addresses.
    pub partition_guid: u128,
    /// Starting LBA of the partition defined by this entry.
    pub start_lba: u64,
    /// Ending LBA of the partition defined by this entry.
    pub end_lba: u64,
    /// Attribute bits, all bits reserved by UEFI
    pub attributes: u64,
    /// Null-terminated string containing a human-readable name of the partition.
    pub name: [u8; 72],
}
assert_eq_size!(Partition, [u8; 128]);

impl for Partition {
/// Returns the partition name as a string
pub fn name() -> Result<String<U36>> {
let mut name: String<U36> = String::new();
for i in 0 .. 72.iter().step_by(2).batching(|mut it| {
match it.next() {
None => None,
Some(x) => match it.next() {
None => None,
Some(y) => u16::from_le_bytes([self.name[x], self.name[y]])
}
}
}) {
if let Some(chr) = from_u32(i) {
name.push(chr)?;
}
}
name
}
}

/// Contains the partition table and the GPT header.
#[derive(Clone, Copy, Debug)]
pub struct PartitionTable<F: fn (u64, u64) -> Vec<u8, U512>> {
    pub header: Header,
    pub partitions: Vec<Partition, U128>
    read_func: f
}

impl PartitionTable {
/// Constructs and reads a partition table, using the given read function to read LBAs.
///
/// The read function is defined as:
///
///```rust
/// fn read_func(lba: u64, count: u64) { ... }
/// ```
///
/// This function is used to read LBAs from a particular storage medium.
/// It should return a Vec<u8, U512> because the GPT header and partition table entries are less than or equal to 512 bytes in length.
pub fn new<F: fn (u64, u64) -> Vec<u8, U512>>(read_func: f) -> Result<self> {
let mut table = PartitionTable {
header: Header::default(),
partitions: Vec::<_, U128>::new(),
read_func: read_func
};
self.read_header()?;
self.read_partition_table()?;
Ok(table)
}

fn read_header(&mut self) -> Result<()> {
    let lba = self.read_func(1, 1);
    if u64::from_le_bytes(lba[0..8]) != GPT_SIG {
        return Err(anyhow!("Not a GPT partition table at bytes 0..7"));
    }
    if checksum_ieee(lba[0..16]) != u32::from_le_bytes(lba[16..20]) {
return Err(anyhow!("IEEE CRC32 checksum {} didn't match checksum {}", checksum_ieee(lba[0..16]), u32::from_le_bytes(lba[16..20])));
}
    self.header = lba[0 .. 92] as Header;
let d1 = u32::from_le_bytes(lba[56 .. 60]);
let d2 = u16::from_le_bytes(lba[60 .. 62]);
let d3 = u16::from_le_bytes(lba[62 .. 64]);
self.disk_guid = uuid::Uuid::from_fields(d1, d2, d3, lba[64 .. 72]).as_u128();
    Ok(())
}

fn read_partition_table(&mut self) ->Result<()> {
for address in self.header.part_entry_lba .. self.header.first_usable_lba {
let lba = self.read_func(address, 1);
for partition_bytes in lba.chunks(header.part_size as usize) {
if u128::from_le_bytes(partition_bytes[0..16]) == 0 {
continue; // partition is unused (might not even be a partition)
}
let partition = partition_bytes as Partition;
// Assemble GUID (as outlined in Appendix A of the UEFI specification, v. 2.8)
{
let d1 = u32::from_le_bytes(partition_bytes[0 .. 4]);
let d2 = u16::from_le_bytes(partition_bytes[4 .. 6]);
let d3 = u16::from_le_bytes(partition_bytes[6 .. 8]);
partition.partition_type_guid = uuid::Uuid::from_fields(d1, d2, d3, partition_bytes[8 .. 16]).as_u128();
}
{
let d1 = u32::from_le_bytes(partition_bytes[16 .. 20]);
let d2 = u16::from_le_bytes(partition_bytes[20 .. 22]);
let d3 = u16::from_le_bytes(partition_bytes[22 .. 24]);
partition.partition_guid = uuid::Uuid::from_fields(d1, d2, d3, partition_bytes[24 .. 32]).as_u128();
}
self.partitions.push(partition)?;
}
}
Ok(())
}
