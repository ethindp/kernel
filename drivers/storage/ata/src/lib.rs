// SPDX-License-Identifier: MPL-2.0
/// The ata library contains modules and functions for reading, writing, and managing devices
/// that comply with the AT Attachment (ATA) standards. This library conforms to
/// [INCITS 529-2018](https://standards.incits.org/apps/group_public/project/details.php?project_id=1643).
///
/// # Features
///
/// The following features can be enabled and disabled to customize what feature sets this
///crate supports. The general, General Purpose Logging (GPL) and Power Management feature
/// sets cannot be disabled because ATA devices must implement these feature sets.
///
/// By default, all features are enabled.
///
/// * lba48: 48-bit Address feature set
/// * amaxaddr: Accessible Max Address Configuration feature set
/// * abo: Advanced Background Operation feature set (ABO)
/// * apm: Advanced Power Management (APM) feature set
/// * dsn: Device Statistics Notification (DSN) feature set
/// * epc: Extended Power Conditions (EPC) feature set
/// * free-fall: Free-fall Control feature set
/// * hybrid-information: Hybrid Information feature set
/// * lls: Long Logical Sector (LLS) feature set
/// * lps: Long Physical Sector (LPS) feature set
/// * ncq: Native Command Queuing (NCQ) feature set
/// * puis: Power-Up In Standby (PUIS) feature set
/// * rebuild-assist: Rebuild Assist feature set
/// * sanitize-device: Sanitize Device feature set
/// * sata-hw-feature-control: SATA Hardware Feature Control Feature Set
/// * security: Security feature set
/// * smart: Self-Monitoring, Analysis, and Reporting Technology (SMART) feature set
/// * sense-data: Sense Data Reporting feature set
/// * ssp: Software Settings Preservation (SSP) feature set
/// * depopulation: Storage Element Depopulation feature set
/// * streaming: Streaming feature set
/// * trusted-computing: Trusted Computing feature set
/// * wrv: Write-Read-Verify feature set
/// * zac: Zoned-device ATA Command Set (ZAC)

/// The General feature set is the base feature set for ATA devices that conform to ATA8 ACS-4.
///
/// The following commands are mandatory for devices that support the General feature set:
///
/// * EXECUTE DEVICE DIAGNOSTIC
/// * IDENTIFY DEVICE
/// * SET FEATURES
///
/// The following commands are optional for devices that support the General feature set:
///
/// * DATA SET MANAGEMENT
/// * DATA SET MANAGEMENT XL
/// * DOWNLOAD MICROCODE
/// * DOWNLOAD MICROCODE DMA
/// * FLUSH CACHE
/// * NOP
/// * READ BUFFER
/// * READ BUFFER DMA
/// * READ DMA
/// * READ SECTOR(S)
/// * READ VERIFY SECTOR(S)
/// * SET DATE & TIME
/// * WRITE BUFFER
/// * WRITE BUFFER DMA
/// * WRITE DMA
/// * WRITE SECTOR(S)
/// * WRITE UNCORRECTABLE EXT
/// * ZERO EXT 
///
/// The following commands are prohibited for devices that support the General feature set:
///
/// * DEVICE RESET
/// * IDENTIFY PACKET DEVICE
/// * PACKET
///
/// The following logs are mandatory for devices that support the General feature set:
///
/// * IDENTIFY DEVICE data log
pub mod general;
/// The General Purpose Logging (GPL) feature set provides access to the logs in a device. These logs are
/// associated with specific feature sets (e.g., the SMART feature set and the Streaming feature set).
/// Support of the individual logs is determined by support of the associated feature set.
///
/// If the device supports a particular feature set, support for any associated log(s) is mandatory.
/// Support for the GPL feature set shall not be disabled by disabling the SMART feature set (see ACS-3). If the
/// feature set associated with a requested log is disabled, the device shall return command aborted.
///
/// If the GPL feature set is supported, the following commands shall be supported:
///
/// * READ LOG EXT
/// * WRITE LOG EXT
///
/// The following commands are optional:
///
/// * READ LOG DMA EXT
/// * WRITE LOG DMA EXT
///
/// If the GPL feature set is supported, all Host Specific logs shall be supported.
pub mod gpl;
/// The Power Management feature set allows a host to modify the behavior of a device in a manner that reduces
/// the power required to operate. The Power Management feature set provides a set of commands and a timer that
/// enable a device to implement low power consumption modes.
///
/// An ATA device shall support the Power Management feature set.
///
/// The Power Management feature set supported by an ATA device shall include the following:
///
/// * the Standby timer;
/// * CHECK POWER MODE command;
/// * IDLE command;
/// * IDLE IMMEDIATE command;
/// * SLEEP command;
/// * STANDBY command; and
/// * STANDBY IMMEDIATE command
pub mod power_management;
#[no_std]

/// Encapsolates an ATA device.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Device {
bars: [usize; 6],
