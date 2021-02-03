use static_assertions::assert_eq_size;

/// Identify namespace data structure. See sec. 5.15.2.1 of the NVMe specification, revision 1.4a.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdentifyNamespaceResponse {
    /// Namespace size
    pub nsez: u64,
    /// Namespace capabilities
    pub ncap: u64,
    /// Namespace utilization
    pub nuse: u64,
    /// Namespace features
    pub nsfeat: u8,
    /// No. of LBA formats
    pub nlbaf: u8,
    /// Formatted LBA size
    pub flbas: u8,
    /// Metadata capabilities
    pub mc: u8,
    /// End-to-end Data Protection Capabilities
    pub dpc: u8,
    /// End-to-end Data Protection Type Settings
    pub dps: u8,
    /// Namespace Multi-path I/O and Namespace Sharing Capabilities
    pub nmic: u8,
    /// Reservation Capabilities
    pub rescap: u8,
    /// Format Progress Indicator
    pub fpi: u8,
    /// Deallocate Logical Block Features
    pub dlfeat: u8,
    /// Namespace Atomic Write Unit Normal
    pub nawun: u16,
    /// Namespace Atomic Write Unit Power Fail
    pub nawupf: u16,
    /// Namespace Atomic Compare & Write Unit
    pub nacwu: u16,
    /// Namespace Atomic Boundary Size Normal
    pub nabsn: u16,
    /// Namespace Atomic Boundary Offset
    pub nabo: u16,
    /// Namespace Atomic Boundary Size Power Fail
    pub nabspf: u16,
    /// Namespace Optimal I/O Boundary
    pub noiob: u16,
    /// NVM Capacity
    pub nvmcap: u128,
    /// Namespace Preferred Write Granularity
    pub npwg: u16,
    /// Namespace Preferred Write Alignment
    pub npwa: u16,
    /// Namespace Preferred Deallocate Granularity
    pub npdg: u16,
    /// Namespace Preferred Deallocate Alignment
    pub npda: u16,
    /// Namespace Optimal Write Size
    pub nows: u16,
    _rsvd1: [u8; 18],
    /// ANA Group Identifier
    pub anagrpid: u32,
    _rsvd2: [u8; 3],
    /// Namespace attributes
    pub nsattr: u8,
    /// NVM Set Identifier
    pub nvmsetid: u16,
    /// Endurance Group Identifier
    pub endgid: u16,
    /// Namespace Globally Unique Identifier
    pub nsguid: [u8; 16],
    /// IEEE Extended Unique Identifier
    pub eui64: [u8; 8],
    /// LBA Format Support
    pub lbaf: [u32; 16],
    _rsvd3: [u8; 192],
    /// Vendor specific
    pub vs: [u8; 3712],
}
assert_eq_size!(IdentifyNamespaceResponse, [u8; 4096]);

/// Identify controller data structure. See sec. 5.15.2.2 of the NVMe specification, revision 1.4a.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdentifyControllerResponse {
    /// PCI Vendor ID
    pub vid: u16,
    /// PCI Subsystem Vendor ID
    pub ssvid: u16,
    /// Serial Number
    pub sn: [u8; 20],
    /// Model Number
    pub mn: [u8; 40],
    /// Firmware Revision
    pub fr: [u8; 8],
    /// Recommended Arbitration Burst
    pub rab: u8,
    /// IEEE OUI Identifier
    pub ieee: [u8; 3],
    /// Controller Multi-Path I/O and Namespace Sharing Capabilities
    pub cmic: u8,
    /// Maximum Data Transfer Size
    pub mdts: u8,
    /// Controller ID
    pub cntlid: u16,
    /// Version
    pub ver: u32,
    /// RTD3 Resume Latency
    pub rtd3r: u32,
    /// RTD3 Entry Latency
    pub rtd3e: u32,
    /// Optional Asynchronous Events Supported
    pub oaes: u32,
    /// Controller Attributes
    pub ctratt: u32,
    /// Read Recovery Levels Supported
    pub rrls: u16,
    _rsvd1: [u8; 9],
    /// Controller Type
    pub cntrltype: u8,
    /// FRU Globally Unique Identifier
    pub fguid: [u8; 16],
    /// Command Retry Delay Times
    pub crdt: [u16; 3],
    _rsvd2: [u8; 119],
    /// NVM Subsystem Report
    pub nvmsr: u8,
    /// VPD Write Cycle Information
    pub vwci: u8,
    /// Management Endpoint Capabilities
    pub mec: u8,
    /// Optional Admin Command Support
    pub oacs: u16,
    /// Abort Command Limit
    pub acl: u8,
    /// Asynchronous Event Request Limit
    pub aerl: u8,
    /// Firmware Updates
    pub frmw: u8,
    /// Log Page Attributes
    pub lpa: u8,
    /// Error Log Page Entries
    pub elpe: u8,
    /// Number of Power States Support
    pub npss: u8,
    /// Admin Vendor Specific Command Configuration
    pub avscc: u8,
    /// Autonomous Power State Transition Attributes
    pub apsta: u8,
    /// Warning Composite Temperature Threshold
    pub wctemp: u16,
    /// Critical Composite Temperature Threshold
    pub cctemp: u16,
    /// Maximum Time for Firmware Activation
    pub mtfa: u16,
    /// Host Memory Buffer Preferred Size
    pub hmpre: u32,
    /// Host Memory Buffer Minimum Size
    pub hmmin: u32,
    /// Total NVM Capacity
    pub tnvmcap: u128,
    /// Unallocated NVM Capacity
    pub unvmcap: u128,
    /// Replay Protected Memory Block Support
    pub rpmbs: u32,
    /// Extended Device Self-test Time
    pub edstt: u16,
    /// Device Self-test Options
    pub dsto: u8,
    /// Firmware Update Granularity
    pub fwug: u8,
    /// Keep Alive Support
    pub kas: u16,
    /// Host Controlled Thermal Management Attributes
    pub hctma: u16,
    /// Minimum Thermal Management Temperature
    pub mntmt: u16,
    /// Maximum Thermal Management Temperature
    pub mxtmt: u16,
    /// Sanitize Capabilities
    pub sanicap: u32,
    /// Host Memory Buffer Minimum Descriptor Entry Size
    pub hmminds: u32,
    /// Host Memory Maximum Descriptors Entries
    pub hmmaxd: u16,
    /// NVM Set Identifier Maximum
    pub nsetidmax: u16,
    /// Endurance Group Identifier Maximum
    pub endgidmax: u16,
    /// ANA Transition Time
    pub anatt: u8,
    /// Asymmetric Namespace Access Capabilities
    pub anacap: u8,
    /// ANA Group Identifier Maximum
    pub anagrpmax: u32,
    /// Number of ANA Group Identifiers
    pub nanagrpid: u32,
    /// Persistent Event Log Size
    pub pels: u32,
    _rsvd3: [u8; 156],
    /// Submission Queue Entry Size
    pub sqes: u8,
    /// Completion Queue Entry Size
    pub cqes: u8,
    /// Maximum Outstanding Commands
    pub maxcmd: u16,
    /// Number of Namespaces
    pub nn: u32,
    /// Optional NVM Command Support
    pub oncs: u16,
    /// Fused Operation Support
    pub fuses: u16,
    /// Format NVM Attributes
    pub fna: u8,
    /// Volatile Write Cache
    pub vwc: u8,
    /// Atomic Write Unit Normal
    pub awun: u16,
    /// Atomic Write Unit Power Fail
    pub awupf: u16,
    /// NVM Vendor Specific Command Configuration
    pub nvscc: u8,
    /// Namespace Write Protection Capabilities
    pub nwpc: u8,
    /// Atomic Compare & Write Unit
    pub acwu: u16,
    _rsvd4: u16,
    /// SGL Support
    pub sgls: u32,
    /// Maximum Number of Allowed Namespaces
    pub mnan: u32,
    _rsvd5: [u8; 224],
    /// NVM Subsystem NVMe Qualified Name
    pub subnqn: [u8; 256],
    _rsvd6: [u8; 768],
    /// I/O Queue Command Capsule Supported Size
    pub ioccsz: u32,
    /// I/O Queue Response Capsule Supported Size
    pub iorcsz: u32,
    /// In Capsule Data Offset
    pub icdoff: u16,
    /// Fabrics Controller Attributes
    pub fcatt: u8,
    /// Maximum SGL Data Block Descriptors
    pub msdbd: u8,
    /// Optional Fabric Commands Support
    pub ofcs: u16,
    _rsvd7: [u8; 242],
    /// Power State Descriptors
    pub psd: [[u128; 2]; 32],
    /// Vendor Specific
    pub vs: [u8; 1024],
}
assert_eq_size!(IdentifyControllerResponse, [u8; 4096]);

/// NVM set list. See sec. 5.15.2.5 of the NVM specification, rev. 1.4a.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NVMSetList {
/// Number of identifiers, 0-31.
pub number_of_ids: u8,
_rsvd: [u8; 127],
/// NVM set entries in the list
pub entries: [NVMSetEntry; 31],
}
assert_eq_size!(NVMSetList, [u8; 4096]);

/// NVM set. See sec. 5.15.2.5 of the NVMe specification, rev. 1.4a.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NVMSetEntry {
/// NVM set identifier
pub set_id: u16,
/// Endurance Group Identifier
pub endurance_grp_id: u16,
_rsvd: u32,
/// Random 4 KiB Read Typical
pub random_read_typical: u32,
/// Optimal Write Size
pub opt_write_sz: u32,
/// Total NVM Set Capacity
pub total_nvm_set_cap: u128,
/// Unallocated NVM Set Capacity
pub unalloc_nvm_set_cap: u128,
_rsvd2: [u128; 5],
}
assert_eq_size!(NVMSetEntry, [u8; 128]);

/// Namespace identifier type
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub union NSIdentifier {
/// IEEE Extended Unique Identifier
pub ieee: u64,
/// Namespace Globally Unique Identifier
pub guid: u128,
/// Namespace UUID
pub uuid: u128,
}

/// Namespace Identification Descriptor
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NSIDDescriptor {
/// Namespace Identifier Type
pub nit: u8,
/// Namespace Identifier Length
pub nidl: u8,
_rsvd: u16,
/// Namespace Identifier
pub nid: NSIdentifier,
}

/// Primary Controller Capabilities Structure
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PrimaryControllerCapabilities {
/// Controller Identifier
pub cntlid: u16,
/// Port Identifier
pub portid: u16,
/// Controller Resource Types
pub crt: u8,
_rsvd: [u8; 27],
/// VQ Resources Flexible Total
pub vqfrt: u32,
/// VQ Resources Flexible Assigned
pub vqrfa: u32,
/// VQ Resources Flexible Allocated to Primary
pub vqrfap: u16,
/// VQ Resources Private Total
pub vqprt: u16,
/// VQ Resources Flexible Secondary Maximum
pub vqfrsm: u16,
/// VQ Flexible Resource Preferred Granularity
pub vqgran: u16,
_rsvd2: u128,
/// VI Resources Flexible Total
pub vifrt: u32,
/// VI Resources Flexible Assigned
pub vifra: u32,
/// VI Resources Flexible Allocated to Primary
pub virfap: u16,
/// VI Resources Private Total
pub viprt: u16,
/// VI Resources Flexible Secondary Maximum
pub vifrsm: u16,
/// VI Flexible Resource Preferred Granularity
pub vigran: u16,
_rsvd3: [u8; 4016],
}
assert_eq_size!(PrimaryControllerCapabilities, [u8; 4096]);

/// Secondary controller entry
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SCEntry {
/// Secondary Controller Identifier
pub scid: u16,
/// Primary Controller Identifier
pub pcid: u16,
/// Secondary Controller State
pub scs: u8,
_rsvd: [u8; 3],
/// Virtual Function Number
pub vfn: u16,
/// Number of VQ Flexible Resources Assigned
pub nvq: u16,
/// Number of VI Flexible Resources Assigned
pub nvi: u16,
_rsvd2: [u8; 17],
}

/// Secondary Controller List
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SCList {
/// Number of identifiers in this list
pub len: u8,
_rsvd: [u8; 31],
/// SC entries
pub entries: [SCEntry; 128],
}
assert_eq_size!(SCList, [u8; 4096]);

/// Namespace granularity list
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NSGranularityList {
/// Namespace Granularity Attributes
pub attrs: u32,
/// Number of Descriptors
pub len: u8,
_rsvd: [u8; 27],
pub descriptors: [NGDescriptor; 16],
}
assert_eq_size!(NSGranularityList, [u8; 288]);

/// Namespace granularity descriptor
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NGDescriptor {
/// Namespace Size Granularity
pub size: u64,
/// Namespace Capacity Granularity
pub capacity: u64,
}
assert_eq_size!(NGDescriptor, [u8; 16]);

/// UUID List
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UUIDList {
_rsvd: u32,
/// List of UUIDs
pub uuids: [UUIDEntry; 128],
}
assert_eq_size!(UUIDList, [u8; 4096]);

/// UUID entry
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UUIDEntry {
/// UUID Lists Entry Header
pub header: u8,
_rsvd: [u8; 16],
/// UUID value
pub uuid: u128,
}
assert_eq_size!(UUIDEntry, [u8; 32]);
