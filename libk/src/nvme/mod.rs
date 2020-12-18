mod queues;
mod structs;
use crate::{
    interrupts::register_interrupt_handler,
    memory::{allocate_phys_range, free_range},
    pci::PCIDevice,
};
use bit_field::BitField;
use core::mem;
use dia_semver::Semver;
use log::*;
use spin::Mutex;
use voladdress::{VolAddress, VolBlock};
use x86::random;
use minivec::MiniVec;
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptStackFrameValue;

lazy_static! {
    static ref CONTROLLERS: Mutex<MiniVec<NVMeController>> = Mutex::new(MiniVec::new());
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
pub struct NVMeController {
    version: Semver,
    /// Controller Capabilities
    cap: u64,
    /// Version
    vs: u64,
    /// Interrupt Mask Set
    intms: u64,
    /// Interrupt Mask Clear
    intmc: u64,
    /// Controller Configuration
    cc: u64,
    /// Controller Status
    csts: u64,
    /// NVM Subsystem Reset (Optional)
    nssr: u64,
    /// Admin Queue Attributes
    aqa: u64,
    /// Admin Submission Queue Base Address
    asq: u64,
    /// Admin Completion Queue Base Address
    acq: u64,
    /// Controller Memory Buffer Location (Optional)
    cmbloc: u64,
    /// Controller Memory Buffer Size (Optional)
    cmbsz: u64,
    /// Boot Partition Information (Optional)
    bpinfo: u64,
    /// Boot Partition Read Select (Optional)
    bprsel: u64,
    /// Boot Partition Memory Buffer Location (Optional)
    bpmbl: u64,
    /// Controller Memory Buffer Memory Space Control (Optional)
    cmbmsc: u64,
    /// Controller Memory Buffer Status (Optional)
    cmbsts: u64,
    /// Persistent Memory Capabilities (Optional)
    pmrcap: u64,
    /// Persistent Memory Region Control (Optional)
    pmrctl: u64,
    /// Persistent Memory Region Status (Optional)
    pmrsts: u64,
    /// Persistent Memory Region Elasticity Buffer Size (optional)
    pmrebs: u64,
    /// Persistent Memory Region Sustained Write Throughput
    pmrswtp: u64,
    /// Persistent Memory Region Controller Memory Space Control (Optional)
    pmrmsc: u64,
    /// Submission Queue 0 Tail Doorbell (Admin)
    adm_sub_tail_queue_doorbell: u64,
    /// Completion Queue 0 Head Doorbell (Admin)
    adm_comp_head_queue_doorbell: u64,
    /// Submission queue tail doorbells
    sub_tail_queue_doorbells: MiniVec<u64>,
    /// Completion queue head doorbells
    comp_head_queue_doorbells: MiniVec<u64>,
    /// 16KiB ring buffer for controller writes
    resp_buffer_addr: u64,
    cqs: MiniVec<queues::CompletionQueue>,
    sqs: MiniVec<queues::SubmissionQueue>,
    resps: MiniVec<MiniVec<queues::CompletionQueueEntry>>,
    intline: u8,
}

impl NVMeController {
/// Initializes:
/// * The NVMe memory addresses for this controller; and
/// * The output buffer (16KiB) for this controller.
///
/// # Safety
///
/// This function is unsafe because the PCI device passed in must be the correct one, or
/// undefined behavior may result.
    pub unsafe fn new(device: PCIDevice) -> Self {
        let mut dev = Self {
            cap: device.bars.0,
            vs: device.bars.0  + 0x08,
            intms: device.bars.0 + 0x0C,
            intmc: device.bars.0 + 0x10,
            cc: device.bars.0 + 0x14,
            csts: device.bars.0 + 0x1C,
            nssr: device.bars.0 + 0x20,
            aqa: device.bars.0 + 0x24,
            asq: device.bars.0 + 0x28,
            acq: device.bars.0 + 0x30,
            cmbloc: device.bars.0 + 0x38,
            cmbsz: device.bars.0 + 0x3C,
            bpinfo: device.bars.0 + 0x40,
            bprsel: device.bars.0 + 0x44,
            bpmbl: device.bars.0 + 0x48,
            cmbmsc: device.bars.0 + 0x50,
            cmbsts: device.bars.0  + 0x58,
            pmrcap: device.bars.0 + 0xE00,
            pmrctl: device.bars.0 + 0xE04,
            pmrsts: device.bars.0 + 0xE08,
            pmrebs: device.bars.0 + 0xE0C,
            pmrswtp: device.bars.0 + 0xE10,
            pmrmsc: device.bars.0 + 0xE14,
            adm_sub_tail_queue_doorbell: device.bars.0 + 0x1000,
            adm_comp_head_queue_doorbell: device.bars.0 + 0x1003, // This isn't correct, but we'll reallocate it
            sub_tail_queue_doorbells: MiniVec::new(),
            comp_head_queue_doorbells: MiniVec::new(),
            version: Semver::new(0, 0, 0),
            intline: device.int_line,
            resp_buffer_addr: 0,
            cqs: MiniVec::new(),
            sqs: MiniVec::new(),
            resps: MiniVec::new(),
        };
        allocate_phys_range(device.bars.0, 0x1003);
        let stride = dev.read_cap().get_bits(32..36);
        dev.adm_comp_head_queue_doorbell =
            device.bars.0  + (0x1003 + (4 << stride));
        allocate_phys_range(device.bars.0 + 0x1000 + (4 << stride), device.bars.0 + 0x1003 + (4 << stride));
        let mut buf_loc = 0u64;
        random::rdrand64(&mut buf_loc);
        buf_loc.set_bits(47..64, 0);
        allocate_phys_range(buf_loc, buf_loc + 16384);
        dev.resp_buffer_addr = buf_loc;
        dev.init();
        dev
    }

#[inline]
pub fn read_cap(&self) -> u64 {
let mem: VolAddress<u64> = unsafe {
VolAddress::new(self.cap as usize)
};
mem.read()
}

#[inline]
pub fn read_vs(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.vs as usize)
};
mem.read()
}

#[inline]
pub fn read_intms(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.intms as usize)
};
mem.read()
}

#[inline]
pub fn read_intmc(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.intmc as usize)
};
mem.read()
}

#[inline]
pub fn write_intmc(&mut self, val: u32) {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.intmc as usize)
};
mem.write(val)
}

#[inline]
pub fn read_cc(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.cc as usize)
};
mem.read()
}

#[inline]
pub fn write_cc(&mut self, val: u32) {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.cc as usize)
};
mem.write(val)
}

#[inline]
pub fn read_csts(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.csts as usize)
};
mem.read()
}

#[inline]
pub fn read_nssr(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.nssr as usize)
};
mem.read()
}

#[inline]
pub fn write_nssr(&mut self, val: u32) {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.nssr as usize)
};
mem.write(val)
}

#[inline]
pub fn read_aqa(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.aqa as usize)
};
mem.read()
}

#[inline]
pub fn write_aqa(&mut self, val: u32) {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.aqa as usize)
};
mem.write(val)
}

#[inline]
pub fn read_asq(&self) -> u64 {
let mem: VolAddress<u64> = unsafe {
VolAddress::new(self.asq as usize)
};
mem.read()
}

#[inline]
pub fn write_asq(&mut self, val: u64) {
let mem: VolAddress<u64> = unsafe {
VolAddress::new(self.aqa as usize)
};
mem.write(val)
}

#[inline]
pub fn read_acq(&self) -> u64 {
let mem: VolAddress<u64> = unsafe {
VolAddress::new(self.acq as usize)
};
mem.read()
}

#[inline]
pub fn write_acq(&mut self, val: u64) {
let mem: VolAddress<u64> = unsafe {
VolAddress::new(self.cc as usize)
};
mem.write(val)
}

#[inline]
pub fn read_cmbloc(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.cmbloc as usize)
};
mem.read()
}

#[inline]
pub fn write_cmbloc(&mut self, val: u32) {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.cmbloc as usize)
};
mem.write(val)
}

#[inline]
pub fn read_cmbsz(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.cmbsz as usize)
};
mem.read()
}

#[inline]
pub fn write_cmbsz(&mut self, val: u32) {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.cmbsz as usize)
};
mem.write(val)
}

#[inline]
pub fn read_bpinfo(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.bpinfo as usize)
};
mem.read()
}

#[inline]
pub fn write_bpinfo(&mut self, val: u32) {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.bpinfo as usize)
};
mem.write(val)
}

#[inline]
pub fn read_bprsel(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.bprsel as usize)
};
mem.read()
}

#[inline]
pub fn write_bprsel(&mut self, val: u32) {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.bprsel as usize)
};
mem.write(val)
}

#[inline]
pub fn read_bpmbl(&self) -> u64 {
let mem: VolAddress<u64> = unsafe {
VolAddress::new(self.bpmbl as usize)
};
mem.read()
}

#[inline]
pub fn write_bpmbl(&mut self, val: u64) {
let mem: VolAddress<u64> = unsafe {
VolAddress::new(self.bpmbl as usize)
};
mem.write(val)
}

#[inline]
pub fn read_cmbmsc(&self) -> u64 {
let mem: VolAddress<u64> = unsafe {
VolAddress::new(self.cmbmsc as usize)
};
mem.read()
}

#[inline]
pub fn write_cmbmsc(&mut self, val: u64) {
let mem: VolAddress<u64> = unsafe {
VolAddress::new(self.cmbmsc as usize)
};
mem.write(val)
}

#[inline]
pub fn read_cmbsts(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.cmbsts as usize)
};
mem.read()
}

#[inline]
pub fn read_pmrcap(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.pmrcap as usize)
};
mem.read()
}

#[inline]
pub fn read_pmrctl(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.pmrctl as usize)
};
mem.read()
}

#[inline]
pub fn write_pmrctl(&mut self, val: u32) {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.pmrctl as usize)
};
mem.write(val)
}

#[inline]
pub fn read_pmrsts(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.pmrsts as usize)
};
mem.read()
}

#[inline]
pub fn read_pmrebs(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.pmrebs as usize)
};
mem.read()
}

#[inline]
pub fn read_pmrswtp(&self) -> u32 {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.pmrswtp as usize)
};
mem.read()
}

#[inline]
pub fn read_pmrmsc(&self) -> u64 {
let mem: VolAddress<u64> = unsafe {
VolAddress::new(self.pmrmsc as usize)
};
mem.read()
}

#[inline]
pub fn write_pmrmsc(&mut self, val: u64) {
let mem: VolAddress<u64> = unsafe {
VolAddress::new(self.pmrmsc as usize)
};
mem.write(val)
}

#[inline]
pub fn write_adm_sub_tail_queue_doorbell(&mut self, val: u32) {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.adm_sub_tail_queue_doorbell as usize)
};
mem.write(val)
}

#[inline]
pub fn write_adm_comp_head_queue_doorbell(&mut self, val: u32) {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.adm_comp_head_queue_doorbell as usize)
};
mem.write(val)
}

#[inline]
pub fn write_sub_tail_doorbell(&mut self, doorbell: usize, val: u32) {
if self.sub_tail_queue_doorbells.len() > doorbell {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.sub_tail_queue_doorbells[doorbell] as usize)
};
mem.write(val);
}
}

#[inline]
pub fn write_comp_head_doorbell(&mut self, doorbell: usize, val: u32) {
if self.comp_head_queue_doorbells.len() > doorbell {
let mem: VolAddress<u32> = unsafe {
VolAddress::new(self.comp_head_queue_doorbells[doorbell] as usize)
};
mem.write(val);
}
}

    fn init(&mut self) {
        info!("initializing controller");
        info!("running controller checks");
        info!("Checking controller version");
        let vs = self.read_vs();
        self.version = Semver::new(vs.get_bits(16..32) as u64, vs.get_bits(8..16) as u64, vs.get_bits(0..8) as u64);
        info!("NVMe version: {}", self.version);
        info!("Checking command set support");
        if self.read_cap().get_bit(37) {
            info!("NVM command set supported");
        } else if self.read_cap().get_bit(44) {
            warn!("Controller only supports administrative commands");
        } else if self.read_cap().get_bit(37) && self.read_cap().get_bit(44) {
            info!("Device supports both NVM and admin-only command sets");
        }
        let mpsmin = {
            let min: u32 = 12 + (self.read_cap().get_bits(48..52) as u32);
            2_u64.pow(min)
        };
        if mpsmin == 4096 {
            info!("device supports 4KiB pages");
        } else {
            error!("device does not support 4KiB pages");
            return;
        }
        let mut nvme_error_count = 0;
        'nvme_init: loop {
            if nvme_error_count > 2 {
                error!("Critical controller reset failure; aborting initialization");
                return;
            }
            info!("resetting controller");
            let mut cc = self.read_cc();
            if cc.get_bit(0) {
                cc.set_bit(0, false);
            }
            self.write_cc(cc);
            let mut asqaddr = 0u64;
            let mut acqaddr = 0u64;
            loop {
                if !self.read_csts().get_bit(0) {
                    break;
                }
                if self.read_csts().get_bit(1) {
                    warn!("Fatal controller error; attempting reset");
                    nvme_error_count += 1;
                    continue 'nvme_init;
                }
            }
            info!("reset complete");
            info!("Configuring queues");
            let mut aqa = self.read_aqa();
            if self.read_cap().get_bits(0..16) > 4095 {
                info!(
                    "Max queue entry limit exceeds 4095 (is {}); restricting",
                    self.read_cap().get_bits(0..16)
                );
                aqa.set_bits(16..29, 4095);
                aqa.set_bits(0..12, 4095);
            } else {
                info!(
                    "Max queue entry limit for admin queue is {}",
                    self.read_cap().get_bits(0..16)
                );
                aqa.set_bits(16..28, self.read_cap().get_bits(0..16) as u32);
                aqa.set_bits(0..12, self.read_cap().get_bits(0..16) as u32);
            }
            self.write_aqa(aqa);
            info!("AQA configured; allocating admin queue");
                unsafe {
                    random::rdrand64(&mut asqaddr);
                    random::rdrand64(&mut acqaddr);
                }
                asqaddr.set_bits(0..12, 0);
                asqaddr.set_bits(47..64, 0);
                self.sqs.push(queues::SubmissionQueue::new(
                    asqaddr,
                    aqa.get_bits(16..28) as u16,
                ));
                acqaddr.set_bits(0..12, 0);
                acqaddr.set_bits(47..64, 0);
                self.cqs.push(queues::CompletionQueue::new(
                    asqaddr,
                    aqa.get_bits(0..12) as u16,
                ));
                info!("ASQ located at {:X}", asqaddr);
                self.write_asq(asqaddr);
                info!("ACQ located at {:X}", acqaddr);
                self.write_acq(acqaddr);
                info!("allocating memory for ASQ");
                allocate_phys_range(
                    asqaddr,
                    if self.read_cap().get_bits(0..16) > 4095 {
                        0x3FFC0
                    } else {
                        self.read_cap().get_bits(0..16) - 1
                    });
                info!("Allocating memory for ACQ");
                allocate_phys_range(
                    acqaddr,
                    if self.read_cap().get_bits(0..16) > 4095 {
                        0xFFF0
                    } else {
                        self.read_cap().get_bits(0..16) - 1
                    },
                );
                self.resps.push(MiniVec::with_capacity(if self.read_cap().get_bits(0..16) > 4095 {
0xFFF0
} else {
(self.read_cap().get_bits(0..16) as usize) - 1
                    }));
                    info!("enabling controller");
            let mut cc = self.read_cc();
            cc.set_bits(20..24, 4); // I/O Completion Queue Entry Size, 1 << 4 = 16
            cc.set_bits(16..20, 6); // I/O Submission Queue Entry Size, 1 << 6 = 64
            cc.set_bits(14..16, 0); // Shutdown Notification, 0 = no notification
            cc.set_bits(11..14, 0); // Arbitration Mechanism Selected, 0 = round-robin
            cc.set_bits(7..11, 0); // Memory Page Size, 0 = (2^(12+0)) = 4096
                                   // I/O Command Set Selected
            if self.read_cap().get_bit(37) {
                cc.set_bits(4..7, 0); // 0 = NVM command set
            } else if self.read_cap().get_bit(44) {
                cc.set_bits(4..7, 7); // 7 = Admin command set only
            }
            cc.set_bits(1..4, 0); // reserved
            cc.set_bit(0, true); // Enable
            self.write_cc(cc);
            loop {
                if self.read_csts().get_bit(0) {
                    break 'nvme_init;
                }
                if self.read_csts().get_bit(1) {
                    warn!("Fatal controller error; attempting reset");
                    free_range(
                        asqaddr,
                        if self.read_cap().get_bits(0..16) > 4095 {
                            0x3FFC0
                        } else {
                            self.read_cap().get_bits(0..16) - 1
                        },
                    );
                    free_range(
                        acqaddr,
                        if self.read_cap().get_bits(0..16) > 4095 {
                            0xFFF0
                        } else {
                            self.read_cap().get_bits(0..16) - 1
                        },
                    );
                    nvme_error_count += 1;
                    continue 'nvme_init;
                }
            }
            }
        info!("Controller enabled");
        if self.read_intmc() != 0 {
            info!("Unmasking all interrupts");
            self.write_intmc(0);
        }
        info!("Sending identify command");
            debug!("Creating default entry");
            let mut entry = queues::SubmissionQueueEntry::default();
            entry.cdw0.set_bits(0..8, AdminCommand::Identify as u32); // Opcode
            entry.cdw0.set_bits(8..10, 0); // Fused operation
            entry.cdw0.set_bits(10..14, 0);
            entry.cdw0.set_bits(14..16, 0);
            entry.cdw0.set_bits(16..32, 0);
            debug!("Setting CDW0[8:32] to 0");
            entry.nsid = 0;
            entry.prps[0] = self.resp_buffer_addr;
            entry.operands[0].set_bits(16..31, 0); // Controller Identifier
            entry.operands[0].set_bits(0..8, 1); // Controller or Namespace Structure
            self.sqs[0].queue_command(entry);
            self.write_adm_sub_tail_queue_doorbell(self.sqs[0].get_queue_tail().into());
            info!("Identify command sent, awaiting response");
            loop {
                if self.resps[0].len() > 0 {
                    info!("Identify command returned data");
                    let resp = self.resps[0].remove(0);
                    if resp.status != 0 {
                    error!("Controller returned status {:X} for identify", resp.status);
                    return;
                    }
                    break;
                }
            }
            // Read data structure
            let mut data = [0u8; 4096];
            data.iter_mut()
                .enumerate()
                .for_each(|(i, b)| {
                let mem: VolBlock<u8, 16384> = unsafe {
                VolBlock::new(self.resp_buffer_addr as usize)
                };
                *b = mem.index(i).read();
                mem.index(i).write(0);
                });
            let response: structs::IdentifyControllerResponse = unsafe { mem::transmute(data) };
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

pub fn handle_interrupt(&mut self, _: InterruptStackFrameValue) {
self.cqs.iter_mut().enumerate().for_each(|(i, queue)| queue.read_new_entries(&mut self.resps[i]));
}
}

pub fn init(dev: PCIDevice) {
let mut controllers = CONTROLLERS.lock();
let mut controller = unsafe {
NVMeController::new(dev)
};
register_interrupt_handler(dev.int_line, &|s| controller.handle_interrupt(s));
controllers.push(controller);
}
