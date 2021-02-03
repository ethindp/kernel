mod queues;
/// The structs module contains various structures for NVMe
pub mod structs;
use crate::{
    interrupts::register_interrupt_handler,
    memory::{allocate_phys_range, free_range, get_free_addr, get_aligned_free_addr},
    pci::PCIDevice,
    disk::*,
};
use bit_field::BitField;
use dia_semver::Semver;
use lazy_static::lazy_static;
use log::*;
use minivec::MiniVec;
use spin::{Mutex, RwLock};
use voladdress::{VolAddress, VolBlock};
use heapless::{FnvIndexMap, Vec, consts::*};
use alloc::{boxed::Box, string::String};
use x86_64::instructions::{hlt, interrupts::{disable as disable_interrupts, enable as enable_interrupts}, random::RdRand};
use rand_core::{RngCore, SeedableRng};
use rand_hc::Hc128Rng;
use core::{mem::size_of, sync::atomic::{AtomicUsize, Ordering}};
pub use structs::*;

lazy_static! {
    static ref CONTROLLERS: Mutex<MiniVec<NVMeController>> = Mutex::new(MiniVec::new());
    static ref INTRS: RwLock<FnvIndexMap<u128, AtomicUsize, U65536>> = RwLock::new(FnvIndexMap::new());
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
    DataSetManagement = 0x09,
    Verify = 0x0C,
    ReservationRegister = 0x0D,
    ReservationReport = 0x0E,
    ReservationAcquire = 0x11,
    ReservationRelease = 0x15,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Default)]
struct Request {
qid: usize,
entry: queues::SubmissionQueueEntry
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Response {
qid: usize,
entry: queues::CompletionQueueEntry
}

/// An NVMe controller holds memory addresses to access NVMe hardware.
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
    cqs: MiniVec<queues::CompletionQueue>,
    sqs: MiniVec<queues::SubmissionQueue>,
    intline: u8,
    id: u128,
    rand: Hc128Rng,
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
    pub unsafe fn new(device: PCIDevice) -> Option<Self> {
        let mut dev = Self {
            cap: device.bars.0,
            vs: device.bars.0 + 0x08,
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
            cmbsts: device.bars.0 + 0x58,
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
            cqs: MiniVec::new(),
            sqs: MiniVec::new(),
            id: device.unique_dev_id,
            rand: init_rand(),
        };
        let _ = allocate_phys_range(device.bars.0, device.bars.0 + 0x1003, true);
        let stride = dev.read_cap().get_bits(32..36);
        dev.adm_comp_head_queue_doorbell = device.bars.0 + (0x1003 + (4 << stride));
        let _ = allocate_phys_range(
            device.bars.0 + 0x1000 + (4 << stride),
            device.bars.0 + 0x1003 + (4 << stride),
            true,
        );
        dev.init();
        Some(dev)
    }

    #[inline]
    fn read_cap(&self) -> u64 {
        let mem: VolAddress<u64> = unsafe { VolAddress::new(self.cap as usize) };
        mem.read()
    }

    #[inline]
    fn read_vs(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.vs as usize) };
        mem.read()
    }

    #[inline]
    fn write_intms(&self, val: u32) {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.intms as usize) };
        mem.write(val)
    }

    #[inline]
    fn write_intmc(&mut self, val: u32) {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.intmc as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_cc(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.cc as usize) };
        mem.read()
    }

    #[inline]
    fn write_cc(&mut self, val: u32) {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.cc as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_csts(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.csts as usize) };
        mem.read()
    }

    #[inline]
    fn read_nssr(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.nssr as usize) };
        mem.read()
    }

    #[inline]
    fn write_nssr(&mut self, val: u32) {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.nssr as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_aqa(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.aqa as usize) };
        mem.read()
    }

    #[inline]
    fn write_aqa(&mut self, val: u32) {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.aqa as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_asq(&self) -> u64 {
        let mem: VolAddress<u64> = unsafe { VolAddress::new(self.asq as usize) };
        mem.read()
    }

    #[inline]
    fn write_asq(&mut self, val: u64) {
        let mem: VolAddress<u64> = unsafe { VolAddress::new(self.asq as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_acq(&self) -> u64 {
        let mem: VolAddress<u64> = unsafe { VolAddress::new(self.acq as usize) };
        mem.read()
    }

    #[inline]
    fn write_acq(&mut self, val: u64) {
        let mem: VolAddress<u64> = unsafe { VolAddress::new(self.acq as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_cmbloc(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.cmbloc as usize) };
        mem.read()
    }

    #[inline]
    fn write_cmbloc(&mut self, val: u32) {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.cmbloc as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_cmbsz(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.cmbsz as usize) };
        mem.read()
    }

    #[inline]
    fn write_cmbsz(&mut self, val: u32) {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.cmbsz as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_bpinfo(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.bpinfo as usize) };
        mem.read()
    }

    #[inline]
    fn write_bpinfo(&mut self, val: u32) {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.bpinfo as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_bprsel(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.bprsel as usize) };
        mem.read()
    }

    #[inline]
    fn write_bprsel(&mut self, val: u32) {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.bprsel as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_bpmbl(&self) -> u64 {
        let mem: VolAddress<u64> = unsafe { VolAddress::new(self.bpmbl as usize) };
        mem.read()
    }

    #[inline]
    fn write_bpmbl(&mut self, val: u64) {
        let mem: VolAddress<u64> = unsafe { VolAddress::new(self.bpmbl as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_cmbmsc(&self) -> u64 {
        let mem: VolAddress<u64> = unsafe { VolAddress::new(self.cmbmsc as usize) };
        mem.read()
    }

    #[inline]
    fn write_cmbmsc(&mut self, val: u64) {
        let mem: VolAddress<u64> = unsafe { VolAddress::new(self.cmbmsc as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_cmbsts(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.cmbsts as usize) };
        mem.read()
    }

    #[inline]
    fn read_pmrcap(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.pmrcap as usize) };
        mem.read()
    }

    #[inline]
    fn read_pmrctl(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.pmrctl as usize) };
        mem.read()
    }

    #[inline]
    fn write_pmrctl(&mut self, val: u32) {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.pmrctl as usize) };
        mem.write(val)
    }

    #[inline]
    fn read_pmrsts(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.pmrsts as usize) };
        mem.read()
    }

    #[inline]
    fn read_pmrebs(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.pmrebs as usize) };
        mem.read()
    }

    #[inline]
    fn read_pmrswtp(&self) -> u32 {
        let mem: VolAddress<u32> = unsafe { VolAddress::new(self.pmrswtp as usize) };
        mem.read()
    }

    #[inline]
    fn read_pmrmsc(&self) -> u64 {
        let mem: VolAddress<u64> = unsafe { VolAddress::new(self.pmrmsc as usize) };
        mem.read()
    }

    #[inline]
    fn write_pmrmsc(&mut self, val: u64) {
        let mem: VolAddress<u64> = unsafe { VolAddress::new(self.pmrmsc as usize) };
        mem.write(val)
    }

    #[inline]
    fn write_adm_sub_tail_queue_doorbell(&mut self, val: u32) {
        let mem: VolAddress<u32> =
            unsafe { VolAddress::new(self.adm_sub_tail_queue_doorbell as usize) };
        mem.write(val)
    }

    #[inline]
    fn write_adm_comp_head_queue_doorbell(&mut self, val: u32) {
        let mem: VolAddress<u32> =
            unsafe { VolAddress::new(self.adm_comp_head_queue_doorbell as usize) };
        mem.write(val)
    }

    #[inline]
    fn write_sub_tail_doorbell(&mut self, doorbell: usize, val: u32) {
        if self.sub_tail_queue_doorbells.len() > doorbell {
            let mem: VolAddress<u32> =
                unsafe { VolAddress::new(self.sub_tail_queue_doorbells[doorbell] as usize) };
            mem.write(val);
        }
    }

    #[inline]
    fn write_comp_head_doorbell(&mut self, doorbell: usize, val: u32) {
        if self.comp_head_queue_doorbells.len() > doorbell {
            let mem: VolAddress<u32> =
                unsafe { VolAddress::new(self.comp_head_queue_doorbells[doorbell] as usize) };
            mem.write(val);
        }
    }

    fn init(&mut self) {
        // 1. Verify controller version
        info!("initializing controller");
        info!("running controller checks");
        info!("Checking controller version");
        let vs = self.read_vs();
        self.version = Semver::new(
            vs.get_bits(16..32) as u64,
            vs.get_bits(8..16) as u64,
            vs.get_bits(0..8) as u64,
        );
        info!("NVMe version: {}", self.version);
        info!("Checking command set support");
        if self.read_cap().get_bit(37) {
            info!("NVM command set supported");
        } else if self.read_cap().get_bit(44) {
            warn!("Controller only supports administrative commands");
        } else if self.read_cap().get_bit(37) && self.read_cap().get_bit(44) {
            info!("Device supports both NVM and admin-only command sets");
        }
        // 2. Verify minimum page size matches the system one
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
            // 3. Reset controller
            info!("resetting controller");
            let mut cc = self.read_cc();
            if cc.get_bit(0) {
                cc.set_bit(0, false);
            }
            self.write_cc(cc);
            // 4. Wait for reset to complete
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
            // 5. Configure admin queue
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
            let asqsize = if self.read_cap().get_bits(0..16) > 4095 {
                4096
            } else {
                self.read_cap().get_bits(0..16) + 1
            };
            let acqsize = if self.read_cap().get_bits(0..16) > 4095 {
                4096
            } else {
                self.read_cap().get_bits(0..16) + 1
            };
            let asqaddr = get_aligned_free_addr((size_of::<queues::SubmissionQueueEntry>() as u64)*asqsize, 4096);
            let acqaddr = get_aligned_free_addr((size_of::<queues::CompletionQueueEntry>() as u64)*acqsize, 4096);
            info!("allocating memory for ASQ, {} bytes", (size_of::<queues::SubmissionQueueEntry>() as u64)*asqsize);
            if !allocate_phys_range(asqaddr, asqaddr + asqsize, false) {
                error!("Cannot allocate SQS!");
                return;
            }
            self.sqs
                .push(queues::SubmissionQueue::new(asqaddr, asqsize as u16));
            info!("Allocating memory for ACQ, {} bytes", (size_of::<queues::CompletionQueueEntry>() as u64)*acqsize);
            if !allocate_phys_range(acqaddr, acqaddr + acqsize, false) {
                error!("Cannot allocate CQS!");
                return;
            }
            self.cqs
                .push(queues::CompletionQueue::new(acqaddr, acqsize as u16));
            info!("ASQ located at {:X}", asqaddr);
            self.write_asq(asqaddr);
            info!("ACQ located at {:X}", acqaddr);
            self.write_acq(acqaddr);
            self.resps.push(MiniVec::with_capacity(
                if self.read_cap().get_bits(0..16) > 4095 {
                    0xFFF0
                } else {
                    self.read_cap().get_bits(0..16) as usize
                },
            ));
            // 6. Configure the controller
            info!("enabling controller");
            let mut cc = self.read_cc();
            // A. Configure I/O queue entry size
            cc.set_bits(20..24, 4); // I/O Completion Queue Entry Size, 1 << 4 = 16
            cc.set_bits(16..20, 6); // I/O Submission Queue Entry Size, 1 << 6 = 64
                                    // B. Configure shutdown notification
            cc.set_bits(14..16, 0); // Shutdown Notification, 0 = no notification
                                    // C. Configure arbitration mechanism
            cc.set_bits(11..14, 0); // Arbitration Mechanism Selected, 0 = round-robin
                                    // D. Configure memory page size
            cc.set_bits(7..11, 0); // Memory Page Size, 0 = (2^(12+0)) = 4096
                                   // E. Configure the I/O command set that is to be used
            if self.read_cap().get_bit(37) {
                cc.set_bits(4..7, 0); // 0 = NVM command set
            } else if self.read_cap().get_bit(44) {
                cc.set_bits(4..7, 7); // 7 = Admin command set only
            }
            cc.set_bits(1..4, 0); // reserved
                                  // F. Enable the controller
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
                            asqaddr + 0x3FFC0
                        } else {
                            asqaddr + self.read_cap().get_bits(0..16)
                        },
                    );
                    free_range(
                        acqaddr,
                        if self.read_cap().get_bits(0..16) > 4095 {
                            acqaddr + 0xFFF0
                        } else {
                            acqaddr + self.read_cap().get_bits(0..16)
                        },
                    );
                    nvme_error_count += 1;
                    continue 'nvme_init;
                }
            }
        }
        info!("Controller enabled");
        info!("Sending identify command");
        
        let (head, body, _) = unsafe { data.align_to::<structs::IdentifyControllerResponse>() };
        if !head.is_empty() {
        error!("Cannot read response; alignment error");
        return;
        }
        let response = &body[0];
        let sn = response.sn;
        let mn = response.mn;
        let fr = response.fr;
        let sn = String::from_utf8_lossy(&sn);
        let mn = String::from_utf8_lossy(&mn);
        let fr = String::from_utf8_lossy(&fr);
        info!("Detected {} NvM controller, serial no. {}", mn, sn);
        info!("Firmware rev. {}", fr);
        let cntlid = response.cntlid;
        info!("Controller ID is {}", cntlid);
        let rpmbs = response.rpmbs;
        if rpmbs.get_bits(0 .. 3) != 0 {
        info!("Controller supports {} RPMBs and security send/receive", rpmbs.get_bits(0 .. 3));
        if rpmbs.get_bits(3 .. 6) == 0 {
        info!("Authentication method: HMAC SHA-256");
        } else {
        info!("Authentication method is unknown");
        }
        info!("RPMB total size: {} KiB", rpmbs.get_bits(16 .. 24) * 128);
        info!("RPMB access size: {} bytes", (rpmbs.get_bits(24 .. 32) + 1) * 512);
        }
    }

    async fn get_all_responses(&mut self) {
        (0..self.cqs.len()).for_each(|i| {
            let mut entries = MiniVec::new();
            self.cqs[i].read_new_entries(&mut entries);
            self.resps[i].append(&mut entries);
        });
    }
}

impl Disk for NVMeController {
type CommandRequest = Request;
type Response = Response;
type Error = queues::Status;
fn process_command(&mut self, req: CommandRequest) -> Result<Self::Response, Self::Error> {
self.sqs[req.qid].queue_command(req.entry);
if req.qid == 0 {
self.write_adm_sub_tail_queue_doorbell(self.sqs[req.qid].get_queue_tail());
} else {
self.write_sub_tail_doorbell(self.sqs[req.qid].get_queue_tail());
}
{
while INTRS.try_read().is_none() {
hlt();
}
}
disable_interrupts();
let mut i = INTRS.write();
let mut entries: MiniVec<queues::CompletionQueueEntry> = MiniVec::new();
self.cqs[req.qid].read_all_entries(&mut entries);
i[self.id].fetch_sub(1, Ordering::SeqCst);
if req.qid == 0 {
self.write_adm_comp_head_queue_doorbell(self.cqs[req.qid].get_queue_head());
} else {
self.write_comp_head_doorbell(self.cqs[req.qid].get_queue_head());
}
if entries.len() > 1 {
warn!("Retrieved more than one response; returning first");
entries.truncate(1);
}
let entry = entries[0];
enable_interrupts();
if entry.status.sc != 0x00 {
Err(entry.status)
} else {
Ok(Response {
qid: req.qid,
entry,
})
}
}

fn process_commands(&mut self, reqs: MiniVec<CommandRequest>) -> MiniVec<Response> {
reqs.iter().for_each(|req| self.sqs[req.qid].queue_command(req.entry));
reqs.iter().for_each(|req| if req.qid == 0 {
self.write_adm_sub_tail_queue_doorbell(self.sqs[req.qid].get_queue_tail());
} else {
self.write_sub_tail_doorbell(self.sqs[req.qid].get_queue_tail());
});
{
loop {
hlt();
if let Some(i) = INTRS.try_read() {
if i[self.id].load(Ordering::SeqCst) != reqs.len() {
continue;
}
}
}
}
disable_interrupts();
let mut i = INTRS.write();
let mut entries: MiniVec<Response> = MiniVec::with_capacity(65536);
reqs.iter().for_each(|req| {
let mut resps: MiniVec<queues::CompletionQueueEntry> = MiniVec::with_capacity(65536);
self.cqs[req.qid].read_all_entries(&mut resps);
resps.shrink_to_fit();
while let Some(r) = resps.pop() {
entries.push(Response {
qid: req.qid,
entry: r,
});
}
if req.qid == 0 {
self.write_adm_comp_head_queue_doorbell(self.cqs[req.qid].get_queue_head());
} else {
self.write_comp_head_doorbell(self.cqs[req.qid].get_queue_head());
}
});
i[self.id].fetch_sub(reqs.len(), Ordering::SeqCst);
enable_interrupts();
entries
}
}

impl NVMeController {
/// Abort command, see sec. 5.1 of NVMe base spec, rev. 1.4b.
///
/// # Arguments
///
/// * cid: Command ID to abort
/// * sqid: Submission queue ID that the command is in
///
/// # Command completion
///
/// Dword 0 indicates whether the command was aborted. If successful, a completion queue
///entry is posted to either the admin or I/O completion queue with a status of Command Abort
/// Requested before the completion of the abort command is posted to the admin completion
/// queue. The entry of the abort command shall have bit 0 cleared to 0 if the command was
/// aborted; otherwise, it shall be set to one.
///
/// # Additional status codes
///
/// * Abort Command Limit Exceeded (0x03)
pub async fn abort(&mut self, cid: u16, sqid: u16) -> Result<bool, queues::Status> {
match self.process_command(Request {
qid: 0,
entry: queues::SubmissionQueueEntry::new(AdminCommand::Abort as u8, queues::OpType::Independent, queues::OpTransportType::PRP, self.rand.next_u16(), 0, None, [None, None], [Some(0_u32.set_bits(0 .. 16,sqid as u32).set_bits(16 .. 32, cid as u32)), None, None, None, None, None]),
}) {
Some(s) => s.cmdret.get_bit(0),
Err(e) => e,
}
}

/// Asynchronous Event Request command, see sec. 5.2 of NVMe base spec, rev. 1.4b
///
/// # Command completion
///
/// A completion queue entry is posted to the admin completion queue if there is an
/// asynchronous event awaiting processing by host software.
///
/// Dword 0 indicates information about the asynchronous event that is being processed.
///
/// * Bits 31:24 are reserved.
/// * Bits 23:16 indicate the log page that host software must read to clear this event.
/// * Bits 15:08 contain asynchronous event information (defined below).
/// * Bits 07:03 are reserved.
/// * Bits 02:00 contain the type of event being processed: 0 = error, 1 = smart/health
/// status, 2 = notice, 6 = NVM command set specific, 7 = vendor specific; 3-5 are reserved.
///
/// For each event type, the asynchronous event information has any of the following values:
///
/// If the event is an error, then:
///
/// * 0x00: Write to Invalid Doorbell Register
/// * 0x01: Invalid Doorbell Write Value
/// * 0x02: Diagnostic Failure
/// * 0x03: Persistent Internal Error
/// * 0x04: Transient Internal Error
/// * 0x05: Firmware Image Load Error
/// * 0x06-0xff: reserved
///
/// If the event is a smart/health status event, then:
///
/// * 0x00: NVM subsystem reliability has been compromised
/// * 0x01: A temperature is greater than or equal to an over temperature threshold
/// * 0x02: Available spare capacity has fallen below the threshold
/// * 0x03-0xff: reserved
///
/// If the event is a notice event, then:
///
/// * 0x00: Namespace Attribute Changed (either identify namespace data structure or
/// namespace list)
/// * 0x01: Firmware Activation Starting
/// * 0x02: Telemetry Log Changed
/// * 0x03: Asymmetric Namespace Access Change
/// * 0x04: Predictable Latency Event Aggregate Log Change
/// * 0x05: LBA Status Information Alert
/// * 0x06: Endurance Group Event Aggregate Log Page Change
/// * 0x07-0xef: Reserved
/// * 0xf0: Discovery Log Page Change
/// * 0xf1-0xff: Reserved
///
/// To clear this event type, host software must perform one of the following actions
/// depending on the notice type:
///
/// * If 0x00, issue get log page command using changed namespace list log page identifier
/// with Retain Asynchronous Event bit clear
/// * If 0x01, read firmware slot information log page
/// * If 0x02, issue get log page command using Telemetry Controller-Initiated log identifier
/// with Retain Asynchronous Event bit clear
/// * If 0x03, issue get log page command using Asymmetric Namespace Access log identifier
/// with Retain Asynchronous Event bit clear
/// * If 0x05, issue get log page command using LBA Status Information log identifier with
/// Retain Asynchronous Event bit clear
/// * If 0x06, issue get log page command using Endurance Group Event Aggregate log
/// identifier with Retain Asynchronous Event bit clear
/// * If 0xf0, read discovery log pages
///
/// If the event is NVM Command Set Specific, then:
///
/// * 0x00: Reservation Log Page Available
/// * 0x01: Sanitize Operation Completed
/// * 0x02: Sanitize Operation Completed With Unexpected Deallocation
/// * 0x03-0xff: Reserved
///
/// If the event is vendor specific, then event information is vendor specific.
///
/// # Additional status codes
///
/// * Asynchronous Event Request Limit Exceeded (0x05)
pub async fn async_event_request(&mut self) -> Result<u32, queues::Status> {
match process_command(Request {
qid: 0,
entry: queues::SubmissionQueueEntry::new(AdminCommand::AsynchronousEventRequest as u8, queues::OpType::Independent, queues::OpTransportType::PRP, self.rand.next_u16(), 0, None, [None, None], [None, None, None, None, None, None]),
}) {
Some(s) => s.cmdret,
Err(e) => e,
}
}

/// Create I/O completion queue command, see sec. 5.3 of the NVMe base specification, rev. 1.4b
///
/// # Arguments
///
/// * PRP Entry 1 (prp1): if pc is true, then this is a 64-bit base memory address pointer of the
/// completion queue that is physically contiguous; otherwise, this is a PRP list that
/// specifies the list of pages that constitute the completion queue. In either case, the PRP
/// offset shall be 0x00 and shall be memory paged aligned as set in CC.MPS. This parameter
/// is optional. If None, an address will automatically be chosen.
/// * Queue size (qsize): specifies the size of this queue. If this is 0x00 or larger than
/// the maximum size that the controller supports this function shall respond with an invalid
/// queue size error.
/// * Queue identifier (qid): specifies the identifier for this queue. This identifier
/// corresponds to the completion queue head doorbell used for this command. This shall not
/// exceed the number of queues feature. If this is 0x00, if it exceeds the number of queues,
/// or if it is already in use, this function shall return Invalid Queue Identifier.
/// * Interrupt vector (iv): specifies the interrupt vector to be utilized for this queue.
/// This is only applicable if using MSI-X or multiple message MSI and should be 0 if using
/// single message MSI or pin-based interrupts. For MSI-X this shall not exceed 2048 nor
/// shall it exceed the number of interrupt vectors the controller supports. If it exceeds
/// the number of vectors the controller supports, this function shall return an invalid
/// interrupt vector error.
/// * Interrupts enabled (ien): determines whether interrupts are enabled for this queue. If
/// false, normal command processing will not function correctly.
/// * Physically contiguous (pc): if true, then the queue is physically contiguous and prp1
/// is the address of a contiguous memory buffer in host memory; if false, then this queue is
/// not physically contiguous and prp1 points to a PRP list. If this is false and CAP.CQR is
/// cleared to 0, this function shall return an invalid field in command error. If the queue
/// is located in the controller memory buffer, pc is false, and CMBLOC.CQPDS is cleared to
/// 0, then this function shall return an invalid use of controller memory buffer error.
///
/// Note: though this function will automatically determine an address in host memory for
/// the queue, it will not allocate the memory that the queue requires. The caller is
/// responsible for memory allocation and deallocation of completion queues.
///
/// # Command completion
///
/// If successful, a completion queue entry shall be posted to the admin completion queue.
///
/// # Additional status codes
///
/// * Invalid Queue Identifier (0x01)
/// * Invalid Queue Size (0x02)
/// * Invalid Interrupt Vector (0x08)
pub async fn create_io_completion_queue(&mut self, prp1: Option<u64>, qsize: u16, qid: u16, iv: u16, ien: bool, pc: bool) -> Result<(), queues::Status> {
match self.process_command(Request {
qid: 0,
entry: queues::SubmissionQueueEntry::new(AdminCommand::CreateIoSubmissionQueue as u8, queues::OpType::Independent, queues::OpTransferType::PRP, self.rand.next_u16(), 0, None, [if let Some (prp) = prp1 {
Some(prp)
} else {
Some(get_aligned_free_addr((size_of::<queues::CompletionQueueEntry>() as u64)*qsize, 4096))
}), None], [Some(0u32.set_bits(16 .. 32, qsize as u32).set_bits(0 .. 16, qid as u32)), Some(0u32.set_bits(16 .. 32, iv as u32).set_bit(1, ien).set_bit(0, pc)), None, None, None, None]),
}) {
Ok(_) => {
self.cqs.push(queues::CompletionQueue::new(entry.prps[0], qsize));
Ok(())
},
Err(e) => e,
}
}

/// Create I/O Submission Queue command, see sec. 5.4 ofNVMe base specification, rev. 1.4b
///
/// # Arguments
///
/// * PRP Entry 1 (prp1): if pc is true, then this is a 64-bit base memory address pointer of the
/// submission queue that is physically contiguous; otherwise, this is a PRP list that
/// specifies the list of pages that constitute the submission queue. In either case, the PRP
/// offset shall be 0x00 and shall be memory paged aligned as set in CC.MPS. This parameter
/// is optional. If None, an address will automatically be chosen.
/// * Queue size (qsize): specifies the size of this queue. If this is 0x00 or larger than
/// the maximum size that the controller supports this function shall respond with an invalid
/// queue size error.
/// * Queue identifier (qid): specifies the identifier for this queue. This identifier
/// corresponds to the submission queue tail doorbell used for this command. This shall not
/// exceed the number of queues feature. If this is 0x00, if it exceeds the number of queues,
/// or if it is already in use, this function shall return Invalid Queue Identifier.
/// * Completion queue identifier (CQID): specifies the completion queue that this
/// submission queue shall be linked to. Command completion queue entries that are generated
/// in response to submission queue entries placed into this queue shall be placed in the
/// completion queue identifier indicated by this parameter. If the value is 0x00
/// (indicating the ACQ) or is outside the range supported by the controller, this function
/// shall return an invalid queue identifier error. If the CQID is within the range
/// supported by the controller but doesn't identify an existing completion queue, this
/// function shall return a completion queue invalid error.
/// * Queue priority (qprio): only used if the weighted round robin with urgent priority
/// class arbitration mechanism is selected during controller initialization, ignored
/// otherwise. This parameter specifies the priority class of this submission queue. The
/// class can either be urgent, high, medium or low.
/// * Physically contiguous (pc): if true, then the queue is physically contiguous and prp1
/// is the address of a contiguous memory buffer in host memory; if false, then this queue is
/// not physically contiguous and prp1 points to a PRP list. If this is false and CAP.CQR is
/// cleared to 0, this function shall return an invalid field in command error. If the queue
/// is located in the controller memory buffer, pc is false, and CMBLOC.CQPDS is cleared to
/// 0, then this function shall return an invalid use of controller memory buffer error.
/// * NVM set identifier (nvmsetid): this parameter indicates the NVM set to be associated
/// with this submission queue.
///
/// Note: though this function will automatically determine an address in host memory for
/// the queue, it will not allocate the memory that the queue requires. The caller is
/// responsible for memory allocation and deallocation of completion queues.
///
/// # Command completion
///
/// Upon completion of this command, the controller posts a completion queue entry to the ACQ.
///
/// # Additional status codes
///
/// * Completion queue invalid (0x00)
/// * Invalid queue identifier (0x01)
/// * Invalid queue size, invalid field in command (0x02)
pub async fn create_io_submission_queue(&mut self, prp1: Option<u64>, qsize: u16, qid: u16, cqid: u16, qprio: QueuePriority, pc: bool, nvmsetid: u16) -> Result<(), queues::Status> {
match self.process_command(Request {
qid: 0,
entry: queues::SubmissionQueueEntry::new(AdminCommand::CreateIoSubmissionQueue as u8, queues::OpType::Independent, queues::OpTransportType::PRP, self.rand.next_u16(), 0, None, [if let Some (prp) = prp1 {
Some(prp)
} else {
Some(get_aligned_free_addr((size_of::<queues::SubmissionQueueEntry>() as u64)*qsize, 4096))
}), None], [Some(0u32.set_bits(16 .. 32, qsize as u32).set_bits(0 .. 16, qid as u32)), Some(0u32.set_bits(16 .. 32, cqid as u32).set_bits(1 .. 3, qprio as u32).set_bit(0, pc)), Some(0u32.set_bits(0 .. 16, nvmsetid as u32)), None, None, None]),
}) {
Ok(_) => {
self.sqs.push(queues::SubmissionQueue::new(entry.prps[0], qsize));
Ok(())
},
Err(e) => e,
}
}

/// Delete I/O completion queue command, see sec. 5.5 of the NVMe base specification, rev. 1.4b
///
/// # Arguments
///
/// * qid: The I/O completion queue to delete. You cannot delete the admin completion queue.
/// You must delete all submission queues associated with this qid before issuing this
/// command.
///
/// # Command completion
///
/// Upon completion, the controller shall post a completion queue entry to the ACQ. Host
/// software may deallocate the memory used by the queue specified by qid after this command
/// has completed.
/// 
/// # Additional status codes
///
/// * Invalid queue identifier (0x01)
/// * Invalid queue deletion (0x0c)
pub async fn delete_io_completion_queue(&mut self, qid: u16) -> Result<(), queues::Status> {
match process_command(Request {
qid: 0,
entry: queues::SubmissionQueueEntry::new(AdminCommand::DeleteIoCompletionQueue as u8, queues::OpType::Independent, queues::OpTransferType::PRP, self.rand.next_u16(), 0, None, [None, None], [Some(qid as u32), None, None, None, None, None]),
}) {
Ok(_) => Ok(()),
Err(e) => Err(e)
}
}

/// Delete I/O submission queue command, see sec. 5.6 of NVMe base specification, rev. 1.4b
///
/// # Arguments
///
/// * Queue identifier (qid): specifies the submission queue to delete. You cannot delete
/// the admin submission queue.
///
/// # Command completion
///
/// After all commands submitted to the indicated I/O Submission Queue are either completed
/// or aborted, a completion queue entry is posted to the Admin Completion Queue when the
/// queue has been deleted.
///
/// # Additional status codes
///
/// * Invalid queue identifier (0x01)
pub async fn delete_io_submission_queue(&mut self, qid: u16) -> Result<(), queues::Status> {
match process_command(Request {
qid: 0,
entry: queues::SubmissionQueueEntry::new(AdminCommand::DeleteIoSubmissionQueue as u8, queues::OpType::Independent, queues::OpTransferType::PRP, self.rand.next_u16(), 0, None, [None, None], [Some(qid as u32), None, None, None, None, None]),
}) {
Ok(_) => Ok(()),
Err(e) => Err(e)
}
}

/// Doorbell Buffer Config command, see sec. 5.7 of NVMe base specification, rev. 1.4b
///
/// # Arguments
///
/// * PRP Entry 1 (prp1): specifies a 64-bit memory address that shall be the shadow doorbell buffer as defined in fig. 164 of the NVMe base specification. The shadow doorbell buffer is updated by host software and shall be memory page aligned.
/// * PRP Entry 2 (prp2): specifies a 64-bit memory address that shall be the base of the EventIdx register as defined in fig. 164 of the NvMe base specification. The EventIdx buffer is updated by the paravirtualized controller and shall be memory page aligned.
///
/// # Command completion
///
/// When the command is completed, the controller posts a completion queue entry to the Admin Completion Queue indicating the status for the command. If the Shadow Doorbell buffer or EventIdx buffer memory addresses are invalid, then a status code of Invalid Field in Command shall be returned.
pub async fn doorbell_buffer_config(&mut self, prp1: u64, prp2: u64) -> Result<(), queues::Status> {
match self.process_command(Request {
qid: 0,
entry: queues::SubmissionQueueEntry::new(AdminCommand::DoorbellBufferConfig as u8, queues::OpType::Independent, queues::OpTransferType::PRP, self.rand.next_u16(), 0, None, [Some(prp1), Some(prp2)], [None; 6]),
}) {
Ok(_) => Ok(()),
Err(e) => Err(e)
}
}

/// Device self-test command
///
/// # Arguments
///
/// * Self-test code (STC): specifies the action to be taken. The code can be one of:
///     * `0x01`: Start a short device self-test operation
///     * `0x02`: start an extended device self-test operation
///     * `0x0e`: start a vendor-specific device self-test operation
///     * `0x0f`: abort active self-test operation
/// * Namespace identifier (NSID): specifies the namespace to include in this self-test. If
/// the value is `0x00000000`, no namespaces shall be included. If the value is between the
/// range [`0x00000001`,`0xfffffffe`], the specified namespace shall be included in the
/// self-test. If the value is `0xffffffff` (`u32::MAX`), all namespaces shall be included
/// in the self-test.
///
/// # Command completion
///
/// A completion queue entry is posted to the Admin Completion Queue after the appropriate
/// actions are taken.
///
/// # Additional status codes
///
/// * Device self-test in progress (`0x1d`)
pub async fn device_self_test(stc: SelfTestCode, nsid: u32) -> Result<(), queues::Status> {
match self.process_command(Request {
qid: 0,
entry: queues::SubmissionQueueEntry::new(AdminCommand::DeviceSelfTest as u8, queues::OpType::Independent, queues::OpTransferType::PRP, self.rand.next_u16(), nsid, None, [None; 2], [Some(stc as u32), None, None, None, None, None]),
}) {
Ok(_) => Ok(()),
Err(e) => Err(e)
}
}


/// Identify command, see sec. 5.15 of NVMe base specification, rev. 1.4b`
///
/// # Arguments
///
/// * Data pointer (dptr): specifies a PRP for the data returned by this command. Only one
/// PRP is required and it mustn't cross a page boundary.
/// * Controller identifier (cntid): specifies the controller identifier used by some
/// identify operations. Whether this is actually used depends on the operation: it is used
/// in the attached controller list, controller list for those controllers in the NVM
/// subsystem, and primary and secondary controller capabilities lists. It may be used in
/// future CNS definitions.
/// * Controller or namespace structure (CNS): specifies the information to return to the
/// host.
/// * NVM set identifier (nvmsetid): specifies the identifier for the NVM set. Only used for
/// the NVM set list.
/// * UUID index (uuid): index of a UUID in the UUID list. Bit 7 is ignored. Optional
///
/// # CNS values
///
/// * `0x00`: identify namespace data structure (requires NSID)
/// * `0x01`: identify controller data structure
/// * `0x02`: active NSID list (requires NSID)
/// * `0x03`: namespace identification descriptor list (requires NSID)
/// * `0x04`: NVM set list (optional, requires NVM set identifier)
/// * `0x10`: allocated namespace ID list (optional, requires NSID)
/// * `0x11`: identify namespace data structure for given allocated NSID (optional, requires NSID)
/// * `0x12`: list of controllers attached to the specified NSID (optional, requires NSID and CNTID)
/// * `0x13`: list of controllers that exist in the NVM subsystem (optional, requires CNTID)
/// * `0x14`: primary controller capabilities data structure for specified primary controller (optional, requires CNTID)
/// * `0x15`: list of controllers associated with the primary controller processing the command (optional, requires CNTID)
/// * `0x16`: namespace granularity list (optional)
/// * `0x17`: UUID list (optional)
///
/// CNS values marked as `optional` may not be supported by this controller.
///
/// # Command completion
///
/// Upon completion of the Identify command, the controller posts a completion queue entry to the Admin Completion Queue.
pub async fn identify(&mut self, nsid: u32, cntid: u16, cns: u8, nvmsetid: u16, uuid: u8) -> Result<IdentifyResponse, queues::Status> {

}

pub fn init(dev: PCIDevice) {
    info!(
        "Registering interrupt handler for interrupt {}",
        dev.int_line
    );
    register_interrupt_handler(dev.int_line, Box::new(move |_| QUEUES.write()[dev.unique_dev_id].fetch_add(1, Ordering::SeqCst)));
    {
    info!("Creating queue");
    let mut queues = QUEUES.write();
    queues.insert(dev.unique_dev_id, AtomicUsize::new(0));
    }
    let mut controllers = CONTROLLERS.lock();
    let controller = unsafe { NVMeController::new(dev) };
    if let Some(c) = controller {
        controllers.push(c);
    } else {
        error!("Cannot add NVMe controller");
        return;
    }
}

fn init_rand() -> Hc128Rng {
let mut seed: Vec<u8, U32> = Vec::new();
let rand = RdRand::new().unwrap();
let mut count = 0;
while let Some(i) = rand.get_u64() {
let bytes = u64::to_le_bytes(i);
count += 8;
if count == 32 {
break;
}
seed.extend_from_slice(&bytes).unwrap();
}
let seed: [u8; 32] = seed.into();
Hc128Rng::from_seed(seed)
}

/// This enumeration holds all possible return values for the identify command.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IdentifyResponse {
/// Identify Namespace data structure for the specified NSID or the common namespace
/// capabilities
IdNamespace(IdentifyNamespaceResponse),
/// Identify Controller data structure for the controller processing the command
IdController(IdentifyControllerResponse),
/// Active Namespace ID list
ActiveNSList([u32; 1024]),
/// Namespace Identification Descriptor list
NSDescList(MiniVec<NSIDDescriptor>),
/// NVM set list
NVMSetList(NVMSetList),
/// Allocated Namespace ID list
AllocNsList([u32; 1024]),
/// Identify Namespace data structure for an Allocated Namespace ID
AllocIdNamespace(IdentifyNamespaceResponse),
/// Namespace Attached Controller list
NsAttachedControllerList([u16; 2048]),
/// Controller list
ControllerList([u16; 2048]),
/// Primary Controller Capabilities data Structure
PrimaryControllerCapabilities(PrimaryControllerCapabilities),
/// Secondary Controller list
ScList(SCList),
/// Namespace Granularity List
NsGranList(NSGranularityList),
/// UUID List
UuidList(UUIDList),
/// Anything else
Other([u8; 4096]),
}

/// If the weighted round robin with urgent priority class arbitration mechanism is supported, then host software
/// may assign a queue priority service class of Urgent, High, Medium, or Low. If the weighted round robin with
/// urgent priority class arbitration mechanism is not supported, then the priority setting is not used and is
/// ignored by the controller.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum QueuePriority {
/// Urgent priority
Urgent,
/// High priority
High,
/// Medium priority
Medium,
/// Low priority
Low,
}

