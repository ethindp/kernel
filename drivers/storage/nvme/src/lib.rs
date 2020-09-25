#![no_std]
use bit_field::BitField;
use core::mem;
use core::sync::atomic::{AtomicBool, Ordering};
use heapless::consts::*;
use heapless::spsc::Queue;
use heapless::Vec;
use lazy_static::lazy_static;
use log::*;
use spin::RwLock;
use static_assertions::assert_eq_size;
use voladdress::{DynamicVolBlock, VolAddress, VolBlock};
use x86::random;

lazy_static! {
    static ref CQS: RwLock<Vec<CompletionQueue, U32>> = RwLock::new(Vec::new());
    static ref SQS: RwLock<Vec<SubmissionQueue, U32>> = RwLock::new(Vec::new());
}

static INTR: AtomicBool = AtomicBool::new(false);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
struct SubmissionQueueEntry {
    pub cdw0: u32,
    pub nsid: u32,
    _rsvd: [u32; 2],
    pub mptr: u64,
    pub prps: [u64; 2],
    pub operands: [u32; 6],
}
assert_eq_size!(SubmissionQueueEntry, [u8; 64]);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
struct CompletionQueueEntry {
    pub cmdret: u32,
    _rsvd: u16,
    pub sqhdptr: u16,
    pub sqid: u16,
    pub cid: u16,
    pub phase: bool,
    pub status: u16,
}
assert_eq_size!(CompletionQueueEntry, [u8; 16]);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
struct SubmissionQueue {
    addr: usize,
    sqh: u16,
    entries: u16,
}

impl SubmissionQueue {
    pub fn new(addr: u64, entries: u16) -> Self {
        SubmissionQueue {
            addr: addr as usize,
            sqh: u16::MAX,
            entries,
        }
    }

    pub fn queue_command(&mut self, entry: SubmissionQueueEntry) {
        let addr: DynamicVolBlock<u32> = unsafe {
            DynamicVolBlock::new(self.addr, (self.entries * 16) as usize)
            };
        self.sqh = self.sqh.wrapping_add(1);
        if self.sqh > self.entries {
            self.sqh = 0;
        }
        // Fill in array
        let mut cmd = [0u32; 16];
        // Dword 0 - CDW0 (command-specific)
        cmd[0] = entry.cdw0;
        // Dword 1 - Namespace ID
        cmd[1] = entry.nsid;
        // Dwords 2-3 reserved
        cmd[2] = 0;
        cmd[3] = 0;
        // Dwords 4-5 - Metadata pointer
        cmd[4] = entry.mptr.get_bits(0..32) as u32;
        cmd[5] = entry.mptr.get_bits(32..64) as u32;
        // Dwords 6-9 - PRP list
        cmd[6] = entry.prps[0].get_bits(0..32) as u32;
        cmd[7] = entry.prps[0].get_bits(32..64) as u32;
        cmd[8] = entry.prps[1].get_bits(0..32) as u32;
        cmd[9] = entry.prps[1].get_bits(32..64) as u32;
        // Dwords 10-15 - command arguments
        cmd[10] = entry.operands[0];
        cmd[11] = entry.operands[1];
        cmd[12] = entry.operands[2];
        cmd[13] = entry.operands[3];
        cmd[14] = entry.operands[4];
        cmd[15] = entry.operands[5];
        for i in 0..16 {
            addr.index((self.sqh as usize) + i).write(cmd[i]);
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
struct CompletionQueue {
    addr: usize,
    cqh: u16,
    entries: u16,
}

impl CompletionQueue {
    pub fn new(addr: u64, entries: u16) -> Self {
        CompletionQueue {
            addr: addr as usize,
            cqh: u16::MAX,
            entries,
        }
    }

    pub fn check_queue_for_new_entries(
        &mut self,
        entry_storage_queue: &mut Queue<CompletionQueueEntry, U65536>,
    ) {
        let addr: DynamicVolBlock<u128> = unsafe { DynamicVolBlock::new(self.addr, self.entries as usize) };
        self.cqh = self.cqh.wrapping_add(1);
        if self.cqh > self.entries {
            self.cqh = 0;
        }
        // Find a new entry with the phase bit set
        // Hopefully this loop should only execute once, but if we need to we loop over the entire
        // queue just in case
        for i in 0..self.entries as usize {
            let entry = addr.index((self.cqh as usize) + i).read();
            if entry.get_bit(112) {
                // New entry; consume it
                let mut cqe = CompletionQueueEntry::default();
                cqe.cmdret = entry.get_bits(0..32) as u32;
                cqe.sqhdptr = entry.get_bits(64..80) as u16;
                cqe.sqid = entry.get_bits(80..96) as u16;
                cqe.cid = entry.get_bits(96..112) as u16;
                cqe.phase = entry.get_bit(112);
                cqe.status = entry.get_bits(113..128) as u16;
                let _ = entry_storage_queue.enqueue(cqe);
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdentifyNamespaceResponse {
    // Namespace size
    pub nsez: u64,
    // Namespace capabilities
    pub ncap: u64,
    // Namespace utilization
    pub nuse: u64,
    // Namespace features
    pub nsfeat: u8,
    // No. of LBA formats
    pub nlbaf: u8,
    // Formatted LBA size
    pub flbas: u8,
    // Metadata capabilities
    pub mc: u8,
    // End-to-end Data Protection Capabilities
    pub dpc: u8,
    // End-to-end Data Protection Type Settings
    pub dps: u8,
    // Namespace Multi-path I/O and Namespace Sharing Capabilities
    pub nmic: u8,
    // Reservation Capabilities
    pub rescap: u8,
    // Format Progress Indicator
    pub fpi: u8,
    // Deallocate Logical Block Features
    pub dlfeat: u8,
    // Namespace Atomic Write Unit Normal
    pub nawun: u16,
    // Namespace Atomic Write Unit Power Fail
    pub nawupf: u16,
    // Namespace Atomic Compare & Write Unit
    pub nacwu: u16,
    // Namespace Atomic Boundary Size Normal
    pub nabsn: u16,
    // Namespace Atomic Boundary Offset
    pub nabo: u16,
    // Namespace Atomic Boundary Size Power Fail
    pub nabspf: u16,
    // Namespace Optimal I/O Boundary
    pub noiob: u16,
    // NVM Capacity
    pub nvmcap: u128,
    // Namespace Preferred Write Granularity
    pub npwg: u16,
    // Namespace Preferred Write Alignment
    pub npwa: u16,
    // Namespace Preferred Deallocate Granularity
    pub npdg: u16,
    // Namespace Preferred Deallocate Alignment
    pub npda: u16,
    // Namespace Optimal Write Size
    pub nows: u16,
    _rsvd1: [u8; 18],
    // ANA Group Identifier
    pub anagrpid: u32,
    _rsvd2: [u8; 3],
    // Namespace attributes
    pub nsattr: u8,
    // NVM Set Identifier
    pub nvmsetid: u16,
    // Endurance Group Identifier
    pub endgid: u16,
    // Namespace Globally Unique Identifier
    pub nsguid: [u8; 16],
    // IEEE Extended Unique Identifier
    pub eui64: [u8; 8],
    // LBA Format Support
    pub lbaf: [u32; 16],
    _rsvd3: [u8; 192],
    // Vendor specific
    pub vs: [u8; 3711],
}
assert_eq_size!(IdentifyNamespaceResponse, [u8; 4096]);

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdentifyControllerResponse {
    // PCI Vendor ID
    pub vid: u16,
    // PCI Subsystem Vendor ID
    pub svid: u16,
    // Serial Number
    pub sn: [u8; 20],
    // Model Number
    pub mn: [u8; 40],
    // Firmware Revision
    pub fr: [u8; 8],
    // Recommended Arbitration Burst
    pub rab: u8,
    // IEEE OUI Identifier
    pub ieee: [u8; 3],
    // Controller Multi-Path I/O and Namespace Sharing Capabilities
    pub cmic: u8,
    // Maximum Data Transfer Size
    pub mdts: u8,
    // Controller ID
    pub cntlid: u16,
    // Version
    pub ver: u32,
    // RTD3 Resume Latency
    pub rtd3r: u32,
    // RTD3 Entry Latency
    pub rtd3e: u32,
    // Optional Asynchronous Events Supported
    pub oaes: u32,
    // Controller Attributes
    pub ctratt: u32,
    // Read Recovery Levels Supported
    pub rrls: u16,
    _rsvd1: [u8; 9],
    // Controller Type
    pub cntrltype: u8,
    // FRU Globally Unique Identifier
    pub fguid: [u8; 16],
    // Command Retry Delay Times
    pub crdt: [u16; 3],
    _rsvd2: [u8; 119],
    // NVM Subsystem Report
    pub nvmsr: u8,
    // VPD Write Cycle Information
    pub vwci: u8,
    // Management Endpoint Capabilities
    pub mec: u8,
    // Optional Admin Command Support
    pub oacs: u16,
    // Abort Command Limit
    pub acl: u8,
    // Asynchronous Event Request Limit
    pub aerl: u8,
    // Firmware Updates
    pub frmw: u8,
    // Log Page Attributes
    pub lpa: u8,
    // Error Log Page Entries
    pub elpe: u8,
    // Number of Power States Support
    pub npss: u8,
    // Admin Vendor Specific Command Configuration
    pub avscc: u8,
    // Autonomous Power State Transition Attributes
    pub apsta: u8,
    // Warning Composite Temperature Threshold
    pub wctemp: u16,
    // Critical Composite Temperature Threshold
    pub cctemp: u16,
    // Maximum Time for Firmware Activation
    pub mtfa: u16,
    // Host Memory Buffer Preferred Size
    pub hmpre: u32,
    // Host Memory Buffer Minimum Size
    pub hmmin: u32,
    // Total NVM Capacity
    pub tnvmcap: u128,
    // Unallocated NVM Capacity
    pub unvmcap: u128,
    // Replay Protected Memory Block Support
    pub rpmbs: u32,
    // Extended Device Self-test Time
    pub edstt: u16,
    // Device Self-test Options
    pub dsto: u8,
    // Firmware Update Granularity
    pub fwug: u8,
    // Keep Alive Support
    pub kas: u16,
    // Host Controlled Thermal Management Attributes
    pub hctma: u16,
    // Minimum Thermal Management Temperature
    pub mntmt: u16,
    // Maximum Thermal Management Temperature
    pub mxtmt: u16,
    // Sanitize Capabilities
    pub sanicap: u32,
    // Host Memory Buffer Minimum Descriptor Entry Size
    pub hmminds: u32,
    // Host Memory Maximum Descriptors Entries
    pub hmmaxd: u16,
    // NVM Set Identifier Maximum
    pub nsetidmax: u16,
    // Endurance Group Identifier Maximum
    pub endgidmax: u16,
    // ANA Transition Time
    pub anatt: u8,
    // Asymmetric Namespace Access Capabilities
    pub anacap: u8,
    // ANA Group Identifier Maximum
    pub anagrpmax: u32,
    // Number of ANA Group Identifiers
    pub nanagrpid: u32,
    // Persistent Event Log Size
    pub pels: u32,
    _rsvd3: [u8; 156],
    // Submission Queue Entry Size
    pub sqes: u8,
    // Completion Queue Entry Size
    pub cqes: u8,
    // Maximum Outstanding Commands
    pub maxcmd: u16,
    // Number of Namespaces
    pub nn: u32,
    // Optional NVM Command Support
    pub oncs: u16,
    // Fused Operation Support
    pub fuses: u16,
    // Format NVM Attributes
    pub fna: u8,
    // Volatile Write Cache
    pub vwc: u8,
    // Atomic Write Unit Normal
    pub awun: u16,
    // Atomic Write Unit Power Fail
    pub awupf: u16,
    // NVM Vendor Specific Command Configuration
    pub nvscc: u8,
    // Namespace Write Protection Capabilities
    pub nwpc: u8,
    // Atomic Compare & Write Unit
    pub acwu: u16,
    _rsvd4: u16,
    // SGL Support
    pub sgls: u32,
    // Maximum Number of Allowed Namespaces
    pub mnan: u32,
    _rsvd5: [u8; 224],
    // NVM Subsystem NVMe Qualified Name
    pub subnqn: [u8; 256],
    _rsvd6: [u8; 768],
    // I/O Queue Command Capsule Supported Size
    pub ioccsz: u32,
    // I/O Queue Response Capsule Supported Size
    pub iorcsz: u32,
    // In Capsule Data Offset
    pub icdoff: u16,
    // Fabrics Controller Attributes
    pub fcatt: u8,
    // Maximum SGL Data Block Descriptors
    pub msdbd: u8,
    // Optional Fabric Commands Support
    pub ofcs: u16,
    _rsvd7: [u8; 242],
    // Power State Descriptors
    pub psd: [[u128; 2]; 32],
    // Vendor Specific
    pub vs: [u8; 1024],
}
assert_eq_size!(IdentifyControllerResponse, [u8; 4096]);

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum AdminCommand {
    DeleteIoSubmissionQueue = 0x00,
    CreateIoSubmissionQueue = 0x01,
    GetLogPage = 0x02,
    DeleteIoCompletionQueue = 0x04,
    CreateIoCompletionQueue = 0x05,
    Identify = 0x06,
    Abort = 0x08,
    SetFeatures = 0x09,
    GetFeatures = 0x0A,
    AsynchronousEventRequest = 0x0C,
    NamespaceManagement = 0x0D,
    FirmwareCommit = 0x10,
    FirmwareImageDownload = 0x11,
    DeviceSelfTest = 0x14,
    NamespaceAttachment = 0x15,
    KeepAlive = 0x18,
    DirectiveSend = 0x19,
    DirectiveReceive = 0x1A,
    VirtualizationManagement = 0x1C,
    MiSend = 0x1D,
    MiReceive = 0x1E,
    DoorbellBufferConfig = 0x7C,
    FormatNvm = 0x80,
    SecuritySend = 0x81,
    SecurityReceive = 0x82,
    Sanitize = 0x84,
    GetLbaStatus = 0x86,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum NvmCommand {
    Flush = 0x00,
    Write = 0x01,
    Read = 0x02,
    WriteUncorrectable = 0x04,
    Compare = 0x05,
    WriteZeros = 0x08,
    DatasetManagement = 0x09,
    Verify = 0x0C,
    ReservationRegister = 0x0D,
    ReservationReport = 0x0E,
    ReservationAcquire = 0x11,
    ReservationRelease = 0x15,
}

#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NvMeController {
    bars: [u64; 6],
    /// Controller Capabilities
    pub cap: VolAddress<u64>,
    /// Version
    pub vs: VolAddress<u32>,
    /// Interrupt Mask Set
    pub intms: VolAddress<u32>,
    /// Interrupt Mask Clear
    pub intmc: VolAddress<u32>,
    /// Controller Configuration
    pub cc: VolAddress<u32>,
    /// Controller Status
    pub csts: VolAddress<u32>,
    /// NVM Subsystem Reset (Optional)
    pub nssr: VolAddress<u32>,
    /// Admin Queue Attributes
    pub aqa: VolAddress<u32>,
    /// Admin Submission Queue Base Address
    pub asq: VolAddress<u64>,
    /// Admin Completion Queue Base Address
    pub acq: VolAddress<u64>,
    /// Controller Memory Buffer Location (Optional)
    pub cmbloc: VolAddress<u32>,
    /// Controller Memory Buffer Size (Optional)
    pub cmbsz: VolAddress<u32>,
    /// Boot Partition Information (Optional)
    pub bpinfo: VolAddress<u32>,
    /// Boot Partition Read Select (Optional)
    pub bprsel: VolAddress<u32>,
    /// Boot Partition Memory Buffer Location (Optional)
    pub bpmbl: VolAddress<u64>,
    /// Controller Memory Buffer Memory Space Control (Optional)
    pub cmbmsc: VolAddress<u64>,
    /// Controller Memory Buffer Status (Optional)
    pub cmbsts: VolAddress<u32>,
    /// Persistent Memory Capabilities (Optional)
    pub pmrcap: VolAddress<u32>,
    /// Persistent Memory Region Control (Optional)
    pub pmrctl: VolAddress<u32>,
    /// Persistent Memory Region Status (Optional)
    pub pmrsts: VolAddress<u32>,
    /// Persistent Memory Region Elasticity Buffer Size (optional)
    pub pmrebs: VolAddress<u32>,
    /// Persistent Memory Region Sustained Write Throughput
    pub pmrswtp: VolAddress<u32>,
    /// Persistent Memory Region Controller Memory Space Control (Optional)
    pub pmrmsc: VolAddress<u64>,
    /// Submission Queue 0 Tail Doorbell (Admin)
    adm_sub_queue_doorbell: VolAddress<u32>,
    /// Completion Queue 0 Head Doorbell (Admin)
    adm_comp_queue_doorbell: VolAddress<u32>,
    /// Submission queue tail doorbells
    sub_queue_doorbells: Vec<VolAddress<u32>, U65536>,
    /// Completion queue head doorbells
    comp_queue_doorbells: Vec<VolAddress<u32>, U65536>,
    /// Memory allocator function; returns nothing but is passed the address and a size
    malloc: fn(u64, u64),
    /// Memory free function; receives an address and a size
    free: fn(u64, u64),
    /// Interrupt registration routine (IRR)
    irr: fn(u8, fn()),
    iline: u8,
}

impl NvMeController {
    pub unsafe fn new(
        bars: [u64; 6],
        iline: u8,
        malloc: fn(u64, u64),
        free: fn(u64, u64),
        irr: fn(u8, fn()),
    ) -> Self {
        let mut dev = Self {
            bars: bars,
            cap: VolAddress::new((bars[0] as usize) + 0x00),
            vs: VolAddress::new((bars[0] as usize) + 0x08),
            intms: VolAddress::new((bars[0] as usize) + 0x0C),
            intmc: VolAddress::new((bars[0] as usize) + 0x10),
            cc: VolAddress::new((bars[0] as usize) + 0x14),
            csts: VolAddress::new((bars[0] as usize) + 0x1C),
            nssr: VolAddress::new((bars[0] as usize) + 0x20),
            aqa: VolAddress::new((bars[0] as usize) + 0x24),
            asq: VolAddress::new((bars[0] as usize) + 0x28),
            acq: VolAddress::new((bars[0] as usize) + 0x30),
            cmbloc: VolAddress::new((bars[0] as usize) + 0x38),
            cmbsz: VolAddress::new((bars[0] as usize) + 0x3C),
            bpinfo: VolAddress::new((bars[0] as usize) + 0x40),
            bprsel: VolAddress::new((bars[0] as usize) + 0x44),
            bpmbl: VolAddress::new((bars[0] as usize) + 0x48),
            cmbmsc: VolAddress::new((bars[0] as usize) + 0x50),
            cmbsts: VolAddress::new((bars[0] as usize) + 0x58),
            pmrcap: VolAddress::new((bars[0] as usize) + 0xE00),
            pmrctl: VolAddress::new((bars[0] as usize) + 0xE04),
            pmrsts: VolAddress::new((bars[0] as usize) + 0xE08),
            pmrebs: VolAddress::new((bars[0] as usize) + 0xE0C),
            pmrswtp: VolAddress::new((bars[0] as usize) + 0xE10),
            pmrmsc: VolAddress::new((bars[0] as usize) + 0xE14),
            adm_sub_queue_doorbell: VolAddress::new((bars[0] as usize) + 0x1000),
            adm_comp_queue_doorbell: VolAddress::new((bars[0] as usize) + 0x1003), // This isn't correct, but we'll reallocate it
            sub_queue_doorbells: Vec::new(),
            comp_queue_doorbells: Vec::new(),
            malloc,
            free,
            irr,
            iline,
        };
        (dev.malloc)(bars[0], 0x1003);
        let stride = dev.cap.read().get_bits(32..36);
        dev.adm_comp_queue_doorbell =
            VolAddress::new((bars[0] as usize) + (0x1003 + (1 * (4 << stride))));
        (dev.malloc)(bars[0], 0x1003 + (1 * (4 << stride)));
        dev
    }

    pub async fn init(&self) {
        info!("initializing controller");
        info!("running controller checks");
        info!("Checking controller version");
        if self.vs.read().get_bits(16..32) < 1 && self.vs.read().get_bits(8..16) < 4 {
            error!(
                "version incompatible; required version: 1.4, available version: {}.{}",
                self.vs.read().get_bits(16..32),
                self.vs.read().get_bits(8..16)
            );
            return;
        }
    debug!("VS = {:X}, {:B}", self.vs.read(), self.vs.read());
        info!("Checking command set support");
        if self.cap.read().get_bit(37) {
            info!("NVM command set supported");
        } else if self.cap.read().get_bit(44) {
            warn!("Controller only supports administrative commands");
        } else if self.cap.read().get_bit(37) && self.cap.read().get_bit(44) {
            info!("Device supports both NVM and admin-only command sets");
        } else if self.cap.read().get_bits(37..45) == 0 {
            error!("Controller supports no command sets!");
            return;
        }
        debug!("CSS = {:X}, {:b}", self.cap.read().get_bits(37 .. 45), self.cap.read().get_bits(37 .. 45));
        let mpsmin = {
            let min: u32 = 12 + (self.cap.read().get_bits(48..52) as u32);
            2_u64.pow(min)
        };
        if mpsmin >= 4096 {
            info!("device supports 4KiB pages");
        } else {
            error!("device does not support 4KiB pages");
            return;
        }
        debug!("MPSMIN = {:X}, {:b}", self.cap.read().get_bits(48 .. 52), self.cap.read().get_bits(48 .. 52));
        info!("resetting controller");
        let mut cc = self.cc.read();
        debug!("CC = {:X}, {:b}", self.cc.read(), self.cc.read());
        debug!("CSTS = {:X}, {:b}", self.csts.read(), self.csts.read());
        cc.set_bit(0, false);
        debug!("CC[0] = 0");
        self.cc.write(cc);
        debug!("CC = {:X}, {:b}", self.cc.read(), self.cc.read());
        loop {
            if !self.csts.read().get_bit(0) {
                break;
            }
        }
        debug!("CSTS = {:X}, {:b}", self.csts.read(), self.csts.read());
        info!("reset complete");
        info!("Configuring queues");
        let mut aqa = self.aqa.read();
        debug!("AQA = {:X}, {:b}", self.aqa.read(), self.aqa.read());
        if self.cap.read().get_bits(0..16) > 4095 {
            info!(
                "Max queue entry limit exceeds 4095 (is {}); restricting",
                self.cap.read().get_bits(0..16)
            );
            aqa.set_bits(16..29, 4095);
            aqa.set_bits(0..12, 4095);
        } else {
            info!(
                "Max queue entry limit for admin queue is {}",
                self.cap.read().get_bits(0..16)
            );
            aqa.set_bits(16..28, self.cap.read().get_bits(0..16) as u32);
            aqa.set_bits(0..12, self.cap.read().get_bits(0..16) as u32);
        }
        self.aqa.write(aqa);
        debug!("AQA = {:X}, {:b}", self.aqa.read(), self.aqa.read());
        info!("AQA configured; allocating admin queue");
        {
            let mut sqs = SQS.write();
            let mut cqs = CQS.write();
            let mut asqaddr: u64 = 0;
            let mut acqaddr: u64 = 0;
            unsafe {
                random::rdrand64(&mut asqaddr);
                random::rdrand64(&mut acqaddr);
            }
            asqaddr.set_bits(0..12, 0);
            asqaddr.set_bits(47..64, 0);
            sqs.push(SubmissionQueue::new(asqaddr, aqa.get_bits(16..28) as u16)).unwrap();
            acqaddr.set_bits(0..12, 0);
            acqaddr.set_bits(47..64, 0);
            cqs.push(CompletionQueue::new(asqaddr, aqa.get_bits(0..12) as u16)).unwrap();
            info!("ASQ located at {:X}", asqaddr);
            self.asq.write(asqaddr);
            info!("ACQ located at {:X}", acqaddr);
            self.acq.write(acqaddr);
            debug!("Stored ASQ = {:X}, {:b}; generated ASQ = {:X}, {:b}", self.asq.read(), self.asq.read(), asqaddr, asqaddr);
            debug!("Stored ACQ = {:X}, {:b}; generated ACQ = {:X}, {:b}", self.acq.read(), self.acq.read(), acqaddr, acqaddr);
            info!("allocating memory for ASQ");
            (self.malloc)(
                asqaddr,
                if self.cap.read().get_bits(0..16) > 4095 {
                    0x3FFC0
                } else {
                    self.cap.read().get_bits(0..16) - 1
                },
            );
            info!("Allocating memory for ACQ");
            (self.malloc)(
                acqaddr,
                if self.cap.read().get_bits(0..16) > 4095 {
                    0xFFF0
                } else {
                    self.cap.read().get_bits(0..16) - 1
                },
            );
        }
        info!("enabling controller");
        let mut cc = self.cc.read();
        debug!("CC = {:X}, {:b}", self.cc.read(), self.cc.read());
        debug!("CSTS = {:X}, {:b}", self.csts.read(), self.csts.read());
        cc.set_bit(0, true);
        debug!("CC[0] = 1");
        self.cc.write(cc);
        debug!("CC = {:X}, {:b}", self.cc.read(), self.cc.read());
        loop {
            if self.csts.read().get_bit(0) {
                break;
            }
        }
        debug!("CSTS = {:X}, {:b}", self.csts.read(), self.csts.read());
        info!("Controller enabled");
        if self.intmc.read() != 0 {
            info!("Unmasking all interrupts");
            self.intmc.write(0);
        }
        info!("Registering interrupt handler");
        (self.irr)(self.iline, || {
            INTR.store(true, Ordering::SeqCst);
        });
        info!("Sending identify command");
        {
            let mut sqs = SQS.write();
            let mut entry = SubmissionQueueEntry::default();
            entry.cdw0.set_bits(0..8, AdminCommand::Identify as u32);
            entry.cdw0.set_bits(8..10, 0);
            entry.cdw0.set_bits(10..14, 0);
            entry.cdw0.set_bits(14..16, 0);
            entry.cdw0.set_bits(16..32, 0);
            entry.nsid = 0;
            let mut output: u64 = 0;
            unsafe {
                random::rdrand64(&mut output);
            }
            (self.malloc)(output, output + 4096);
            entry.prps[0] = output;
            entry.operands[0].set_bits(16..31, 0);
            entry.operands[0].set_bits(0..8, 1);
            sqs[0].queue_command(entry);
            info!("Identify command sent, awaiting response");
            loop {
                if INTR.load(Ordering::SeqCst) {
                    info!("Identify command returned data");
                    break;
                }
            }
            INTR.store(false, Ordering::SeqCst);
            // Read data structure
            let mut data = [0u8; 4096];
            {
                let addr: VolBlock<u8, 4096> = unsafe { VolBlock::new(output as usize) };
                for i in 0..4096 {
                    data[i] = addr.index(i).read();
                }
            }
            let response: IdentifyControllerResponse = unsafe { mem::transmute(data) };
            info!(
                "Controller is {}",
                match response.cntrltype {
                    0x00 => "an unknown device",
                    0x01 => "an I/O controller",
                    0x02 => "A discovery controller",
                    0x3 => "Administrative controller",
                    _ => "something else",
                }
            );
        }
    }
}
