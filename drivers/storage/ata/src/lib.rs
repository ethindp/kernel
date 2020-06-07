/// The ata library contains modules and functions for reading, writing, and managing devices that comply with the AT Attachment (ATA) standards. This library conforms to [INCITS 529-2018](https://standards.incits.org/apps/group_public/project/details.php?project_id=1643). It is useful to have the below standards available if you wish to fully understand the ATA architecture:
///
/// * [INCITS 451-2008[R2018]: Information technology - AT Attachments-8 ATA/ATAPI Architecture Model (ATA8-AAM)](https://standards.incits.org/apps/group_public/project/details.php?project_id=2114)
/// * [INCITS 493-2012 [R2017]: Information Technology - AT Attachment-8 - Serial Transport (ATA8-AST)](https://standards.incits.org/apps/group_public/project/details.php?project_id=1830)
/// * [Serial ATA Revision 3.4](https://sata-io.org/developers/purchase-specification)
/// * [INCITS 502-2019: Information technology - SCSI Primary Commands - 5 (SPC-5)](https://standards.incits.org/apps/group_public/project/details.php?project_id=392)
/// * [INCITS 506-202x: Information technology - SBC-4 (SCSI Block Commands - 4)](https://standards.incits.org/apps/group_public/project/details.php?project_id=1780)
/// * [INCITS 537-2016: Information technology – Zoned Device ATA Command Set (ZAC)](https://standards.incits.org/apps/group_public/project/details.php?project_id=403) and its amendment, [INCITS 537-2016/AM 1-2019 - Information technology - Zoned-device ATA Commands Amendment 1 (ZAC-AM1)](https://standards.incits.org/apps/group_public/project/details.php?project_id=2054)
/// * [INCITS 522-2014: Information technology - ATA/ATAPI Command Set - 3 (ACS-3)](https://standards.incits.org/apps/group_public/project/details.php?project_id=1520)
/// * [INCITS 524-2016: Information Technology - AT Attachment 8 - ATA/ATAPI Parallel Transport (ATA8-APT)](https://standards.incits.org/apps/group_public/project/details.php?project_id=373)
///
/// Note: these documents, along with [RFC 3280](https://tools.ietf.org/html/rfc3280), [RFC 3281](https://tools.ietf.org/html/rfc3281), and [SFF-8447](ftp://ftp.seagate.com/sff/SFF-8447.PDF) are listed as normative references in INCITS 529, as well as [ISO 7999:1999](https://www.iso.org/standard/24919.html), [INCITS 4-1986](https://standards.incits.org/apps/group_public/project.php?project_id=1829), [FIPS 140-2](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.140-2.pdf) and [FIPS 140-3](https://csrc.nist.gov/publications/detail/fips/140/3/final). However, not all of these standards are required to utilize this crate successfully, and users do not need to acquire these standards if they are not interested in all of the information (as it can be quite costly).
///
/// Content taken from the above standards will include section numbers, where applicable. These are references to the sections in the standards (e.g.: for feature sets, a reference to section 4.2 would be refering to section 4.2 of ACS-4).
///
/// # Features
///
/// The following features can be enabled and disabled to customize what feature sets this crate supports. The general, General Purpose Logging (GPL) and Power Management feature sets (see sections 4.2, 4.10, and 4.15 of ACS-4) cannot be disabled because ATA devices must implement these feature sets. Section numbers for each feature set are listed in parentheses.
///
/// * lba48: 48-bit Address feature set (4.3)
/// * amaxaddr: Accessible Max Address Configuration feature set (4.4)
/// * abo: Advanced Background Operation feature set (ABO) (4.5)
/// * apm: Advanced Power Management (APM) feature set (4.6)
/// * dsn: Device Statistics Notification (DSN) feature set (4.7)
/// * epc: Extended Power Conditions (EPC) feature set (4.8)
/// * free-fall: Free-fall Control feature set (4.9)
/// * hybrid-information: Hybrid Information feature set (4.11)
/// * lls: Long Logical Sector (LLS) feature set (4.12)
/// * lps: Long Physical Sector (LPS) feature set (4.13)
/// * ncq: Native Command Queuing (NCQ) feature set (4.14)
/// * puis: Power-Up In Standby (PUIS) feature set (4.16)
/// * rebuild-assist: Rebuild Assist feature set (4.17)
/// * sanitize-device: Sanitize Device feature set (4.18)
/// * sata-hw-feature-control: SATA Hardware Feature Control Feature Set (4.19)
/// * security: Security feature set (4.20)
/// * smart: Self-Monitoring, Analysis, and Reporting Technology (SMART) feature set (4.21)
/// * sense-data: Sense Data Reporting feature set (4.22)
/// * ssp: Software Settings Preservation (SSP) feature set (4.23)
/// * depopulation: Storage Element Depopulation feature set (4.24)
/// * streaming: Streaming feature set (4.25)
/// * trusted-computing: Trusted Computing feature set (4.26)
/// * wrv: Write-Read-Verify feature set (4.27)
///
/// # Crate usage
///
/// This crate relies on a few callbacks, which can be set by calling the appropriate functions:
///
/// * dma_in: function to be called to read data using DMA and MMIO
/// * dma_out: function to be called to write data using DMA and MMIO
/// * pio_in: function to be used to read data using Programmed Input/Output (PIO)
/// * pio_out: function to be used to write data using Programmed Input/Output (PIO)
///
/// Each function will be passed a single argument, or no arguments at all, depending on purpose:
///
/// * Input functions will return a DeviceResponse data structure and will be passed the number of bytes to read.
/// * Output functions will be passed a DeviceRequest data structure containing all the required information, and will return nothing
#[no_std]
use bitwise::Word;
use heapless::Vec;
use heapless::consts::*;

#[derive(Clone, Copy, Hash, Eq, partialEq, Ord, PartialOrd, Debug)]
pub enum Feature {
Bit8(u8),
Bit16(u16)
}

#[derive(Clone, Copy, Hash, Eq, partialEq, Ord, PartialOrd, Debug)]
pub enum Count {
Bit9*u8),
Bit16(u16)
}

#[derive(Clone, Copy, Hash, Eq, partialEq, Ord, PartialOrd, Debug)]
pub enum Lba {
Bit28(u32),
Bit48(u64)
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DeviceRequest {
/// Each transport standard defines how the FEATURE field is mapped for proper functionality. Each
/// transport standard also defines how 28-bit commands are mapped differently than 48-bit
/// commands.
pub feature: Feature,
/// Each transport standard defines how the COUNT field is mapped for proper functionality. Each
/// transport standard also defines how 28-bit commands are mapped differently than 48-bit
/// commands.
pub count: Count,
/// For many commands, the LBA field contains the LBA of the first logical sector to be transferred.
/// Each transport standard defines how the LBA field is mapped to the appropriate fields or registers.
pub lba: Lba,
/// Each transport standard defines how the ICC field, if present, is mapped to the appropriate fields
/// or registers. The ICC field is not present in all commands.
pub icc: Option<u8>,
/// Each transport standard defines how the AUXILIARY field, if present, is mapped to the appropriate
/// fields or registers. The AUXILIARY field is not present in all commands.
pub auxiliary: Option<u32>,
/// Each transport standard defines how the DEVICE field bits 7:4 are mapped. Bits 3:0 are marked
/// reserved in every reference to the DEVICE field.
pub device: u8,
/// The COMMAND field contains the command code.
pub command: u8,
/// Optional output data structure (in ACS-4, this section is called "Output from the Host to the Device Data Structure")
pub input: Option<Vec<u8, U512>>
}

