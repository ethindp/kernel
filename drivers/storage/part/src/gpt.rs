// SPDX-License-Identifier: MPL-2.0
extern crate alloc;
use alloc::format;
use anyhow::{anyhow, Result};
use core::char::from_u32;
use core::fmt;
use core::fmt::Debug;
use core::default::Default;
use core::convert::TryInto;
use heapless::consts::*;
use heapless::{String, Vec};
use static_assertions::assert_eq_size;
use zerocopy::{FromBytes, LayoutVerified};

const GPT_SIG: u64 = 0x5452_4150_2049_4645;
const CRC32_TAB: [u32; 256] = [
    0x00000000, 0x77073096, 0xEE0E612C, 0x990951BA, 0x076DC419, 0x706AF48F, 0xE963A535, 0x9E6495A3,
    0x0EDB8832, 0x79DCB8A4, 0xE0D5E91E, 0x97D2D988, 0x09B64C2B, 0x7EB17CBD, 0xE7B82D07, 0x90BF1D91,
    0x1DB71064, 0x6AB020F2, 0xF3B97148, 0x84BE41DE, 0x1ADAD47D, 0x6DDDE4EB, 0xF4D4B551, 0x83D385C7,
    0x136C9856, 0x646BA8C0, 0xFD62F97A, 0x8A65C9EC, 0x14015C4F, 0x63066CD9, 0xFA0F3D63, 0x8D080DF5,
    0x3B6E20C8, 0x4C69105E, 0xD56041E4, 0xA2677172, 0x3C03E4D1, 0x4B04D447, 0xD20D85FD, 0xA50AB56B,
    0x35B5A8FA, 0x42B2986C, 0xDBBBC9D6, 0xACBCF940, 0x32D86CE3, 0x45DF5C75, 0xDCD60DCF, 0xABD13D59,
    0x26D930AC, 0x51DE003A, 0xC8D75180, 0xBFD06116, 0x21B4F4B5, 0x56B3C423, 0xCFBA9599, 0xB8BDA50F,
    0x2802B89E, 0x5F058808, 0xC60CD9B2, 0xB10BE924, 0x2F6F7C87, 0x58684C11, 0xC1611DAB, 0xB6662D3D,
    0x76DC4190, 0x01DB7106, 0x98D220BC, 0xEFD5102A, 0x71B18589, 0x06B6B51F, 0x9FBFE4A5, 0xE8B8D433,
    0x7807C9A2, 0x0F00F934, 0x9609A88E, 0xE10E9818, 0x7F6A0DBB, 0x086D3D2D, 0x91646C97, 0xE6635C01,
    0x6B6B51F4, 0x1C6C6162, 0x856530D8, 0xF262004E, 0x6C0695ED, 0x1B01A57B, 0x8208F4C1, 0xF50FC457,
    0x65B0D9C6, 0x12B7E950, 0x8BBEB8EA, 0xFCB9887C, 0x62DD1DDF, 0x15DA2D49, 0x8CD37CF3, 0xFBD44C65,
    0x4DB26158, 0x3AB551CE, 0xA3BC0074, 0xD4BB30E2, 0x4ADFA541, 0x3DD895D7, 0xA4D1C46D, 0xD3D6F4FB,
    0x4369E96A, 0x346ED9FC, 0xAD678846, 0xDA60B8D0, 0x44042D73, 0x33031DE5, 0xAA0A4C5F, 0xDD0D7CC9,
    0x5005713C, 0x270241AA, 0xBE0B1010, 0xC90C2086, 0x5768B525, 0x206F85B3, 0xB966D409, 0xCE61E49F,
    0x5EDEF90E, 0x29D9C998, 0xB0D09822, 0xC7D7A8B4, 0x59B33D17, 0x2EB40D81, 0xB7BD5C3B, 0xC0BA6CAD,
    0xEDB88320, 0x9ABFB3B6, 0x03B6E20C, 0x74B1D29A, 0xEAD54739, 0x9DD277AF, 0x04DB2615, 0x73DC1683,
    0xE3630B12, 0x94643B84, 0x0D6D6A3E, 0x7A6A5AA8, 0xE40ECF0B, 0x9309FF9D, 0x0A00AE27, 0x7D079EB1,
    0xF00F9344, 0x8708A3D2, 0x1E01F268, 0x6906C2FE, 0xF762575D, 0x806567CB, 0x196C3671, 0x6E6B06E7,
    0xFED41B76, 0x89D32BE0, 0x10DA7A5A, 0x67DD4ACC, 0xF9B9DF6F, 0x8EBEEFF9, 0x17B7BE43, 0x60B08ED5,
    0xD6D6A3E8, 0xA1D1937E, 0x38D8C2C4, 0x4FDFF252, 0xD1BB67F1, 0xA6BC5767, 0x3FB506DD, 0x48B2364B,
    0xD80D2BDA, 0xAF0A1B4C, 0x36034AF6, 0x41047A60, 0xDF60EFC3, 0xA867DF55, 0x316E8EEF, 0x4669BE79,
    0xCB61B38C, 0xBC66831A, 0x256FD2A0, 0x5268E236, 0xCC0C7795, 0xBB0B4703, 0x220216B9, 0x5505262F,
    0xC5BA3BBE, 0xB2BD0B28, 0x2BB45A92, 0x5CB36A04, 0xC2D7FFA7, 0xB5D0CF31, 0x2CD99E8B, 0x5BDEAE1D,
    0x9B64C2B0, 0xEC63F226, 0x756AA39C, 0x026D930A, 0x9C0906A9, 0xEB0E363F, 0x72076785, 0x05005713,
    0x95BF4A82, 0xE2B87A14, 0x7BB12BAE, 0x0CB61B38, 0x92D28E9B, 0xE5D5BE0D, 0x7CDCEFB7, 0x0BDBDF21,
    0x86D3D2D4, 0xF1D4E242, 0x68DDB3F8, 0x1FDA836E, 0x81BE16CD, 0xF6B9265B, 0x6FB077E1, 0x18B74777,
    0x88085AE6, 0xFF0F6A70, 0x66063BCA, 0x11010B5C, 0x8F659EFF, 0xF862AE69, 0x616BFFD3, 0x166CCF45,
    0xA00AE278, 0xD70DD2EE, 0x4E048354, 0x3903B3C2, 0xA7672661, 0xD06016F7, 0x4969474D, 0x3E6E77DB,
    0xAED16A4A, 0xD9D65ADC, 0x40DF0B66, 0x37D83BF0, 0xA9BCAE53, 0xDEBB9EC5, 0x47B2CF7F, 0x30B5FFE9,
    0xBDBDF21C, 0xCABAC28A, 0x53B39330, 0x24B4A3A6, 0xBAD03605, 0xCDD70693, 0x54DE5729, 0x23D967BF,
    0xB3667A2E, 0xC4614AB8, 0x5D681B02, 0x2A6F2B94, 0xB40BBE37, 0xC30C8EA1, 0x5A05DF1B, 0x2D02EF8D,
];

/// The partition table header defines the usable blocks on the disk. It also defines the number and size of the partition entries that make up the partition table.
/// See section 5.3.2 of the UEFI specification, v. 2.8, for extra
/// information on the GPT header and how software should handle it.
#[repr(packed)]
#[derive(Clone, Copy, Debug, Default, FromBytes)]
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
#[repr(C)]
#[derive(Clone, Copy, FromBytes)]
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
    /// Attribute bits, all bits reserved by UEFI.
    pub attributes: u64,
    /// Null-terminated string containing a human-readable name of the partition.
    pub name: [u16; 36],
}
assert_eq_size!(Partition, [u8; 128]);

impl Partition {
    /// Returns the partition name as a string
    pub fn name(self) -> Result<String<U36>> {
        let mut name: String<U36> = String::new();
        for i in 0..36 {
            if let Some(chr) = from_u32(i) {
                if name.push(chr).is_err() {
                return Err(anyhow!("Can't push character {}; out of space", chr));
                }
            }
        }
        Ok(name)
    }
}

impl Debug for Partition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self.name() {
            Ok(n) => n,
            Err(_) => String::new()
            };
        f.debug_struct("Partition")
            .field(
                "partition_type_guid",
                &uuid::Uuid::from_u128(self.partition_type_guid)
                    .to_hyphenated()
                    .encode_upper(&mut uuid::Uuid::encode_buffer()),
            )
            .field(
                "partition_guid",
                &uuid::Uuid::from_u128(self.partition_guid)
                    .to_hyphenated()
                    .encode_upper(&mut uuid::Uuid::encode_buffer()),
            )
            .field("start_lba", &self.start_lba)
            .field("end_lba", &self.end_lba)
            .field("attributes", &self.attributes)
            .field("name", &name.as_str())
            .finish()
    }
}

impl Default for Partition {
fn default() -> Self {
Partition {
partition_type_guid: 0,
partition_guid: 0,
start_lba: 0,
end_lba: 0,
attributes: 0,
name: [0; 36]
}
}
}

/// Contains the partition table and the GPT header.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct PartitionTable {
    pub header: Header,
    pub partitions: Vec<Partition, U128>,
    read_func: fn(u64, u16) -> Vec<u8, U512>,
}

impl PartitionTable {
    /// Constructs and reads a partition table, using the given read function to read LBAs.
    ///
    /// The read function is defined as:
    ///
    ///```rust
    /// fn read_func(lba: u64, count: u16) { ... }
    /// ```
    ///
    /// This function is used to read LBAs from a particular storage medium.
    /// It should return a Vec<u8, U512> because the GPT header and partition table entries are less than or equal to 512 bytes in length.
    pub fn new(read_func: fn(u64, u16) -> Vec<u8, U512>) -> Result<Self> {
        let mut table = PartitionTable {
            header: Header::default(),
            partitions: Vec::<_, U128>::new(),
            read_func,
        };
        table.read_header()?;
        table.read_partition_table()?;
        Ok(table)
    }

    fn read_header(&mut self) -> Result<()> {
        let lba = (self.read_func)(1, 1);
        if u64::from_le_bytes(lba[0..8].try_into().unwrap()) != GPT_SIG {
            return Err(anyhow!("Not a GPT partition table at bytes 0..7"));
        }
        if self.compute_crc32(&lba[0..16]) != u32::from_le_bytes(lba[16..20].try_into().unwrap()) {
            return Err(anyhow!(
                "IEEE CRC32 checksum {} didn't match checksum {}",
                self.compute_crc32(&lba[0..16]),
                u32::from_le_bytes(lba[16..20].try_into().unwrap())
            ));
        }
        self.header = match LayoutVerified::<&[u8], Header>::new(&lba[0 .. 92]) {
        Some(h) => *h,
        None => return Err(anyhow!("Cannot deserialize GPT header"))
        };
        let d1 = u32::from_le_bytes(lba[56..60].try_into().unwrap());
        let d2 = u16::from_le_bytes(lba[60..62].try_into().unwrap());
        let d3 = u16::from_le_bytes(lba[62..64].try_into().unwrap());
        self.header.disk_guid = uuid::Uuid::from_fields(d1, d2, d3, lba[64..72].try_into().unwrap()).unwrap().as_u128();
        // Compute CRC32 of partition array
        let mut crc_bytes: Vec<u8, U0> = Vec::new();
        for address in self.header.part_entry_lba..self.header.first_usable_lba {
            let lba = (self.read_func)(address, 1);
            for partition_bytes in lba.chunks(self.header.part_size as usize) {
                crc_bytes.resize(crc_bytes.len() + partition_bytes.len(), 0).unwrap();
                crc_bytes.extend_from_slice(partition_bytes).unwrap();
            }
        }
        if self.compute_crc32(&crc_bytes[..]) != self.header.partition_entry_array_crc {
        let pe_crc = self.header.partition_entry_array_crc;
            return Err(anyhow!(
                "IEEE CRC32 checksum for partition array {} didn't match checksum {}",
                self.compute_crc32(&crc_bytes[..]),
                pe_crc
            ));
        }
        Ok(())
    }

    fn read_partition_table(&mut self) -> Result<()> {
        for address in self.header.part_entry_lba..self.header.first_usable_lba {
            let lba = (self.read_func)(address, 1);
            for partition_bytes in lba.chunks(self.header.part_size as usize) {
                if u128::from_le_bytes(partition_bytes[0..16].try_into().unwrap()) == 0 {
                    continue; // partition is unused (might not even be a partition)
                }
                let mut partition: Partition = match LayoutVerified::<&[u8], Partition>::new(partition_bytes) {
                Some(p) => *p,
                None => return Err(anyhow!("Cannot decode partition"))
                };
                // Assemble GUID (as outlined in Appendix A of the UEFI specification, v. 2.8)
                {
                    let d1 = u32::from_le_bytes(partition_bytes[0..4].try_into().unwrap());
                    let d2 = u16::from_le_bytes(partition_bytes[4..6].try_into().unwrap());
                    let d3 = u16::from_le_bytes(partition_bytes[6..8].try_into().unwrap());
                    partition.partition_type_guid =
                        uuid::Uuid::from_fields(d1, d2, d3, partition_bytes[8..16].try_into().unwrap()).unwrap().as_u128();
                }
                {
                    let d1 = u32::from_le_bytes(partition_bytes[16..20].try_into().unwrap());
                    let d2 = u16::from_le_bytes(partition_bytes[20..22].try_into().unwrap());
                    let d3 = u16::from_le_bytes(partition_bytes[22..24].try_into().unwrap());
                    partition.partition_guid =
                        uuid::Uuid::from_fields(d1, d2, d3, partition_bytes[24..32].try_into().unwrap()).unwrap().as_u128();
                }
                self.partitions.push(partition).unwrap();
            }
        }
        Ok(())
    }

    fn compute_crc32(&mut self, buf: &[u8]) -> u32 {
        let mut crc32: u32 = !0;
        for i in buf.iter() {
            crc32 = CRC32_TAB[(crc32 ^ *i as u32) as usize & 0xFF] ^ (crc32 >> 8);
        }
        crc32 ^ !0
    }
}
