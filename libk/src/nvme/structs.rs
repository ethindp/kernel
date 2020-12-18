use static_assertions::assert_eq_size;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct IdentifyNamespaceResponse {
    /// Namespace size
    pub(crate) nsez: u64,
    /// Namespace capabilities
    pub(crate) ncap: u64,
    /// Namespace utilization
    pub(crate) nuse: u64,
    /// Namespace features
    pub(crate) nsfeat: u8,
    /// No. of LBA formats
    pub(crate) nlbaf: u8,
    /// Formatted LBA size
    pub(crate) flbas: u8,
    /// Metadata capabilities
    pub(crate) mc: u8,
    /// End-to-end Data Protection Capabilities
    pub(crate) dpc: u8,
    /// End-to-end Data Protection Type Settings
    pub(crate) dps: u8,
    /// Namespace Multi-path I/O and Namespace Sharing Capabilities
    pub(crate) nmic: u8,
    /// Reservation Capabilities
    pub(crate) rescap: u8,
    /// Format Progress Indicator
    pub(crate) fpi: u8,
    /// Deallocate Logical Block Features
    pub(crate) dlfeat: u8,
    /// Namespace Atomic Write Unit Normal
    pub(crate) nawun: u16,
    /// Namespace Atomic Write Unit Power Fail
    pub(crate) nawupf: u16,
    /// Namespace Atomic Compare & Write Unit
    pub(crate) nacwu: u16,
    /// Namespace Atomic Boundary Size Normal
    pub(crate) nabsn: u16,
    /// Namespace Atomic Boundary Offset
    pub(crate) nabo: u16,
    /// Namespace Atomic Boundary Size Power Fail
    pub(crate) nabspf: u16,
    /// Namespace Optimal I/O Boundary
    pub(crate) noiob: u16,
    /// NVM Capacity
    pub(crate) nvmcap: u128,
    /// Namespace Preferred Write Granularity
    pub(crate) npwg: u16,
    /// Namespace Preferred Write Alignment
    pub(crate) npwa: u16,
    /// Namespace Preferred Deallocate Granularity
    pub(crate) npdg: u16,
    /// Namespace Preferred Deallocate Alignment
    pub(crate) npda: u16,
    /// Namespace Optimal Write Size
    pub(crate) nows: u16,
    _rsvd1: [u8; 18],
    /// ANA Group Identifier
    pub(crate) anagrpid: u32,
    _rsvd2: [u8; 3],
    /// Namespace attributes
    pub(crate) nsattr: u8,
    /// NVM Set Identifier
    pub(crate) nvmsetid: u16,
    /// Endurance Group Identifier
    pub(crate) endgid: u16,
    /// Namespace Globally Unique Identifier
    pub(crate) nsguid: [u8; 16],
    /// IEEE Extended Unique Identifier
    pub(crate) eui64: [u8; 8],
    /// LBA Format Support
    pub(crate) lbaf: [u32; 16],
    _rsvd3: [u8; 192],
    /// Vendor specific
    pub(crate) vs: [u8; 3711],
}
assert_eq_size!(IdentifyNamespaceResponse, [u8; 4096]);

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct IdentifyControllerResponse {
    /// PCI Vendor ID
    pub(crate) vid: u16,
    /// PCI Subsystem Vendor ID
    pub(crate) svid: u16,
    /// Serial Number
    pub(crate) sn: [u8; 20],
    /// Model Number
    pub(crate) mn: [u8; 40],
    /// Firmware Revision
    pub(crate) fr: [u8; 8],
    /// Recommended Arbitration Burst
    pub(crate) rab: u8,
    /// IEEE OUI Identifier
    pub(crate) ieee: [u8; 3],
    /// Controller Multi-Path I/O and Namespace Sharing Capabilities
    pub(crate) cmic: u8,
    /// Maximum Data Transfer Size
    pub(crate) mdts: u8,
    /// Controller ID
    pub(crate) cntlid: u16,
    /// Version
    pub(crate) ver: u32,
    /// RTD3 Resume Latency
    pub(crate) rtd3r: u32,
    /// RTD3 Entry Latency
    pub(crate) rtd3e: u32,
    /// Optional Asynchronous Events Supported
    pub(crate) oaes: u32,
    /// Controller Attributes
    pub(crate) ctratt: u32,
    /// Read Recovery Levels Supported
    pub(crate) rrls: u16,
    _rsvd1: [u8; 9],
    /// Controller Type
    pub(crate) cntrltype: u8,
    /// FRU Globally Unique Identifier
    pub(crate) fguid: [u8; 16],
    /// Command Retry Delay Times
    pub(crate) crdt: [u16; 3],
    _rsvd2: [u8; 119],
    /// NVM Subsystem Report
    pub(crate) nvmsr: u8,
    /// VPD Write Cycle Information
    pub(crate) vwci: u8,
    /// Management Endpoint Capabilities
    pub(crate) mec: u8,
    /// Optional Admin Command Support
    pub(crate) oacs: u16,
    /// Abort Command Limit
    pub(crate) acl: u8,
    /// Asynchronous Event Request Limit
    pub(crate) aerl: u8,
    /// Firmware Updates
    pub(crate) frmw: u8,
    /// Log Page Attributes
    pub(crate) lpa: u8,
    /// Error Log Page Entries
    pub(crate) elpe: u8,
    /// Number of Power States Support
    pub(crate) npss: u8,
    /// Admin Vendor Specific Command Configuration
    pub(crate) avscc: u8,
    /// Autonomous Power State Transition Attributes
    pub(crate) apsta: u8,
    /// Warning Composite Temperature Threshold
    pub(crate) wctemp: u16,
    /// Critical Composite Temperature Threshold
    pub(crate) cctemp: u16,
    /// Maximum Time for Firmware Activation
    pub(crate) mtfa: u16,
    /// Host Memory Buffer Preferred Size
    pub(crate) hmpre: u32,
    /// Host Memory Buffer Minimum Size
    pub(crate) hmmin: u32,
    /// Total NVM Capacity
    pub(crate) tnvmcap: u128,
    /// Unallocated NVM Capacity
    pub(crate) unvmcap: u128,
    /// Replay Protected Memory Block Support
    pub(crate) rpmbs: u32,
    /// Extended Device Self-test Time
    pub(crate) edstt: u16,
    /// Device Self-test Options
    pub(crate) dsto: u8,
    /// Firmware Update Granularity
    pub(crate) fwug: u8,
    /// Keep Alive Support
    pub(crate) kas: u16,
    /// Host Controlled Thermal Management Attributes
    pub(crate) hctma: u16,
    /// Minimum Thermal Management Temperature
    pub(crate) mntmt: u16,
    /// Maximum Thermal Management Temperature
    pub(crate) mxtmt: u16,
    /// Sanitize Capabilities
    pub(crate) sanicap: u32,
    /// Host Memory Buffer Minimum Descriptor Entry Size
    pub(crate) hmminds: u32,
    /// Host Memory Maximum Descriptors Entries
    pub(crate) hmmaxd: u16,
    /// NVM Set Identifier Maximum
    pub(crate) nsetidmax: u16,
    /// Endurance Group Identifier Maximum
    pub(crate) endgidmax: u16,
    /// ANA Transition Time
    pub(crate) anatt: u8,
    /// Asymmetric Namespace Access Capabilities
    pub(crate) anacap: u8,
    /// ANA Group Identifier Maximum
    pub(crate) anagrpmax: u32,
    /// Number of ANA Group Identifiers
    pub(crate) nanagrpid: u32,
    /// Persistent Event Log Size
    pub(crate) pels: u32,
    _rsvd3: [u8; 156],
    /// Submission Queue Entry Size
    pub(crate) sqes: u8,
    /// Completion Queue Entry Size
    pub(crate) cqes: u8,
    /// Maximum Outstanding Commands
    pub(crate) maxcmd: u16,
    /// Number of Namespaces
    pub(crate) nn: u32,
    /// Optional NVM Command Support
    pub(crate) oncs: u16,
    /// Fused Operation Support
    pub(crate) fuses: u16,
    /// Format NVM Attributes
    pub(crate) fna: u8,
    /// Volatile Write Cache
    pub(crate) vwc: u8,
    /// Atomic Write Unit Normal
    pub(crate) awun: u16,
    /// Atomic Write Unit Power Fail
    pub(crate) awupf: u16,
    /// NVM Vendor Specific Command Configuration
    pub(crate) nvscc: u8,
    /// Namespace Write Protection Capabilities
    pub(crate) nwpc: u8,
    /// Atomic Compare & Write Unit
    pub(crate) acwu: u16,
    _rsvd4: u16,
    /// SGL Support
    pub(crate) sgls: u32,
    /// Maximum Number of Allowed Namespaces
    pub(crate) mnan: u32,
    _rsvd5: [u8; 224],
    /// NVM Subsystem NVMe Qualified Name
    pub(crate) subnqn: [u8; 256],
    _rsvd6: [u8; 768],
    /// I/O Queue Command Capsule Supported Size
    pub(crate) ioccsz: u32,
    /// I/O Queue Response Capsule Supported Size
    pub(crate) iorcsz: u32,
    /// In Capsule Data Offset
    pub(crate) icdoff: u16,
    /// Fabrics Controller Attributes
    pub(crate) fcatt: u8,
    /// Maximum SGL Data Block Descriptors
    pub(crate) msdbd: u8,
    /// Optional Fabric Commands Support
    pub(crate) ofcs: u16,
    _rsvd7: [u8; 242],
    /// Power State Descriptors
    pub(crate) psd: [[u128; 2]; 32],
    /// Vendor Specific
    pub(crate) vs: [u8; 1024],
}
assert_eq_size!(IdentifyControllerResponse, [u8; 4096]);
