#![no_std]
extern crate alloc;
use bit_field::BitField;
use voladdress::VolAddress;
use log::*;
use x86::random;
use alloc::vec::Vec;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
struct SubmissionQueueEntry {
pub cdw0: u32,
pub nsid: u32,
_rsvd: [u32; 2],
pub mptr: u64,
pub prps: [u64; 2],
pub operands: [u32; 5]
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
struct CompletionQueueEntry {
pub cmdret: u32,
_rsvd: u32,
pub sqhdptr: u16,
pub sqid: u16,
pub cmdid: u16,
pub phase: bool,
pub status: u16,
}

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
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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
sub_queue_doorbells: Vec<VolAddress<u32>>,
/// Completion queue head doorbells
comp_queue_doorbells: Vec<VolAddress<u32>>,
/// Submission queues
sub_queues: Vec<VolAddress<u32>>,
/// Completion Queues
comp_queues: Vec<VolAddress<u128>>,
/// Memory allocator function; returns nothing but is passed the address and a size
malloc: fn (u64, u64),
/// Memory free function; receives an address and a size
free: fn (u64, u64),
}

impl NvMeController {
pub unsafe fn new(bars: [u64; 6], malloc: fn (u64, u64), free: fn (u64, u64)) -> Self {
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
sub_queues: Vec::new(),
comp_queues: Vec::new(),
malloc,
free
};
(dev.malloc)(bars[0] , 0x1003);
let stride = dev.cap.read().get_bits(32 .. 36);
dev.adm_comp_queue_doorbell =VolAddress::new((bars[0] as usize) + (0x1003 + (1 * (4 << stride))));
(dev.malloc)(bars[0], 0x1003 + (1 * (4 << stride)));
dev
}

pub async fn init(&self) {
info!("initializing controller");
info!("running controller checks");
info!("Checking controller version");
if self.vs.read().get_bits(16 .. 32) < 1 || self.vs.read().get_bits(8 .. 16) < 4 {
warn!("version incompatible; required version: {}.{}, available version: {}.{}", 1, 4, self.vs.read().get_bits(16 .. 32), self.vs.read().get_bits(8 .. 16));
warn!("VS field: {} ({:X})", self.vs.read(), self.vs.read());
}
info!("Checking command set support");
if self.cap.read().get_bit(37) {
info!("NVM command set supported");
} else if self.cap.read().get_bit(44) {
warn!("Controller only supports administrative commands");
}
let mpsmin = {
let min: u32 = 12 + (self.cap.read().get_bits(48 .. 52) as u32);
2_u64.pow(min)
};
if mpsmin >= 4096 {
info!("device supports 4KiB pages");
} else {
warn!("device does not support 4KiB pages");
}
info!("resetting controller");
let mut cc = self.cc.read();
cc.set_bit(0, false);
self.cc.write(cc);
loop {
if !self.csts.read().get_bit(0) {
break;
}
}
info!("reset complete");
info!("Configuring queues");
let mut aqa = self.aqa.read();
if self.cap.read().get_bits(0 .. 16) > 4096 {
info!("Max queue entry limit exceeds 4096; restricting");
aqa.set_bits(16 .. 28, 4096);
aqa.set_bits(0 .. 12, 4096);
} else {
info!("Max queue entry limit for admin queue is {}", self.cap.read().get_bits(0 .. 16));
aqa.set_bits(16 .. 28, self.cap.read().get_bits(0 .. 16) as u32);
aqa.set_bits(0 .. 12, self.cap.read().get_bits(0 .. 16) as u32);
}
self.aqa.write(aqa);
info!("AQA configured; allocating admin queue");
let mut asqaddr: u64 = 0;
let mut acqaddr: u64 = 0;
unsafe {
random::rdrand64(&mut asqaddr);
random::rdrand64(&mut acqaddr);
}
asqaddr.set_bits(0 .. 12, 0);
asqaddr.set_bits(48..64, 0);
acqaddr.set_bits(0 .. 12, 0);
acqaddr.set_bits(48..64, 0);
info!("ASQ located at {:X}", asqaddr);
self.asq.write(asqaddr);
info!("ACQ located at {:X}", acqaddr);
self.acq.write(acqaddr);
info!("allocating memory for ASQ");
(self.malloc)(asqaddr, if self.cap.read().get_bits(0 .. 16) > 4096 {
0x40000
} else {
self.cap.read().get_bits(0 .. 16)
});
info!("Allocating memory for ACQ");
(self.malloc)(acqaddr, if self.cap.read().get_bits(0 .. 16) > 4096 {
0x10000
} else {
self.cap.read().get_bits(0 .. 16)
});
info!("enabling controller");
cc.set_bit(0, true);
self.cc.write(cc);
loop {
if self.csts.read().get_bit(0) {
break;
}
}
info!("Controller enabled");
}
}
