#![no_std]
mod queues;
mod structs;
use bit_field::BitField;
use core::mem;
use core::sync::atomic::{AtomicBool, Ordering};
use dia_semver::Semver;
use heapless::consts::*;
use heapless::Vec;
use lazy_static::lazy_static;
use log::*;
use spin::RwLock;
use voladdress::{VolAddress, VolBlock};
use x86::halt;
use x86::random;
use libk::{pci::PCIDevice, memory::{allocate_phys_range, free_range}, interrupts::register_interrupt_handler};

lazy_static! {
    static ref CQS: RwLock<Vec<queues::CompletionQueue, U32>> = RwLock::new(Vec::new());
    static ref SQS: RwLock<Vec<queues::SubmissionQueue, U32>> = RwLock::new(Vec::new());
}

static INTR: AtomicBool = AtomicBool::new(false);

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
    version: Semver,
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
    /// Interrupt line
    intline: u8,
    /// 16KiB ring buffer for controller writes
    resp_buffer: VolBlock<u8, 16384>,
    resp_buffer_addr: u64,
}

impl NvMeController {
    pub unsafe fn new(device: &PCIDevice) -> Self {
        let mut dev = Self {
            bars: device.bars,
            cap: VolAddress::new((device.bars.0 as usize) + 0x00),
            vs: VolAddress::new((device.bars.0 as usize) + 0x08),
            intms: VolAddress::new((device.bars.0 as usize) + 0x0C),
            intmc: VolAddress::new((device.bars.0 as usize) + 0x10),
            cc: VolAddress::new((device.bars.0 as usize) + 0x14),
            csts: VolAddress::new((device.bars.0 as usize) + 0x1C),
            nssr: VolAddress::new((device.bars.0 as usize) + 0x20),
            aqa: VolAddress::new((device.bars.0 as usize) + 0x24),
            asq: VolAddress::new((device.bars.0 as usize) + 0x28),
            acq: VolAddress::new((device.bars.0 as usize) + 0x30),
            cmbloc: VolAddress::new((device.bars.0 as usize) + 0x38),
            cmbsz: VolAddress::new((device.bars.0 as usize) + 0x3C),
            bpinfo: VolAddress::new((device.bars.0 as usize) + 0x40),
            bprsel: VolAddress::new((device.bars.0 as usize) + 0x44),
            bpmbl: VolAddress::new((device.bars.0 as usize) + 0x48),
            cmbmsc: VolAddress::new((device.bars.0 as usize) + 0x50),
            cmbsts: VolAddress::new((device.bars.0 as usize) + 0x58),
            pmrcap: VolAddress::new((device.bars.0 as usize) + 0xE00),
            pmrctl: VolAddress::new((device.bars.0 as usize) + 0xE04),
            pmrsts: VolAddress::new((device.bars.0 as usize) + 0xE08),
            pmrebs: VolAddress::new((device.bars.0 as usize) + 0xE0C),
            pmrswtp: VolAddress::new((device.bars.0 as usize) + 0xE10),
            pmrmsc: VolAddress::new((device.bars.0 as usize) + 0xE14),
            adm_sub_queue_doorbell: VolAddress::new((device.bars.0 as usize) + 0x1000),
            adm_comp_queue_doorbell: VolAddress::new((device.bars.0 as usize) + 0x1003), // This isn't correct, but we'll reallocate it
            sub_queue_doorbells: Vec::new(),
            comp_queue_doorbells: Vec::new(),
            intline: device.int_line,
            version: Semver::new(0, 0, 0),
            resp_buffer: VolBlock::new(0x0),
            resp_buffer_addr: 0,
        };
        allocate_phys_range(device.bars.0, 0x1003);
        let stride = dev.cap.read().get_bits(32..36);
        dev.adm_comp_queue_doorbell =
            VolAddress::new((device.bars.0 as usize) + (0x1003 + (1 * (4 << stride))));
        allocate_phys_range(device.bars.0, 0x1003 + (1 * (4 << stride)));
        let mut buf_loc = 0u64;
        random::rdrand64(&mut buf_loc);
        buf_loc.set_bits(47 .. 64, 0);
        dev.resp_buffer = VolBlock::new(buf_loc as usize);
        allocate_phys_range(buf_loc, buf_loc + 16384);
        dev.resp_buffer_addr = buf_loc;
        dev
    }

    pub async fn init(&mut self) {
        info!("initializing controller");
        info!("running controller checks");
        info!("Checking controller version");
        let vs = self.vs.read();
        self.version = Semver::new(
            vs.get_bits(16..32) as u64,
            vs.get_bits(8..16) as u64,
            vs.get_bits(0..8) as u64,
        );
        info!("NVMe version: {}", self.version);
        info!("Checking command set support");
        if self.cap.read().get_bit(37) {
            info!("NVM command set supported");
        } else if self.cap.read().get_bit(44) {
            warn!("Controller only supports administrative commands");
        } else if self.cap.read().get_bit(37) && self.cap.read().get_bit(44) {
            info!("Device supports both NVM and admin-only command sets");
        }
        let mpsmin = {
            let min: u32 = 12 + (self.cap.read().get_bits(48..52) as u32);
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
            let mut cc = self.cc.read();
            if cc.get_bit(0) {
                cc.set_bit(0, false);
            }
            self.cc.write(cc);
            let mut asqaddr = 0u64;
            let mut acqaddr = 0u64;
            loop {
                if !self.csts.read().get_bit(0) {
                    break;
                }
                if self.csts.read().get_bit(1) {
                    warn!("Fatal controller error; attempting reset");
                    nvme_error_count += 1;
                    continue 'nvme_init;
                }
                unsafe {
                    halt();
                }
            }
            info!("reset complete");
            info!("Configuring queues");
            let mut aqa = self.aqa.read();
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
            info!("AQA configured; allocating admin queue");
            {
                let mut sqs = SQS.write();
                let mut cqs = CQS.write();
                unsafe {
                    random::rdrand64(&mut asqaddr);
                    random::rdrand64(&mut acqaddr);
                }
                asqaddr.set_bits(0..12, 0);
                asqaddr.set_bits(47..64, 0);
                sqs.push(queues::SubmissionQueue::new(
                    asqaddr,
                    aqa.get_bits(16..28) as u16,
                ))
                .unwrap();
                acqaddr.set_bits(0..12, 0);
                acqaddr.set_bits(47..64, 0);
                cqs.push(queues::CompletionQueue::new(
                    asqaddr,
                    aqa.get_bits(0..12) as u16,
                ))
                .unwrap();
                info!("ASQ located at {:X}", asqaddr);
                self.asq.write(asqaddr);
                info!("ACQ located at {:X}", acqaddr);
                self.acq.write(acqaddr);
                info!("allocating memory for ASQ");
                allocate_phys_range(
                    asqaddr,
                    if self.cap.read().get_bits(0..16) > 4095 {
                        0x3FFC0
                    } else {
                        self.cap.read().get_bits(0..16) - 1
                    },
                );
                info!("Allocating memory for ACQ");
                allocate_phys_range(
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
            cc.set_bits(24 .. 32, 0); // Reserved
            cc.set_bits(20 .. 24, 4); // I/O Completion Queue Entry Size, 1 << 4 = 16
            cc.set_bits(16 .. 20, 6); // I/O Submission Queue Entry Size, 1 << 6 = 64
            cc.set_bits(14 .. 16, 0); // Shutdown Notification, 0 = no notification
            cc.set_bits(11 .. 14, 0); // Arbitration Mechanism Selected, 0 = round-robin
            cc.set_bits(7 .. 11, 0); // Memory Page Size, 0 = (2^(12+0)) = 4096
            // I/O Command Set Selected
            if self.cap.read().get_bit(37) {
                cc.set_bits(4..7, 0); // 0 = NVM command set
            } else if self.cap.read().get_bit(44) {
                cc.set_bits(4..7, 7); // 7 = Admin command set only
            }
            cc.set_bits(1 .. 4, 0); // reserved
            cc.set_bit(0, true); // Enable
            self.cc.write(cc);
            loop {
                if self.csts.read().get_bit(0) {
                    break 'nvme_init;
                }
                if self.csts.read().get_bit(1) {
                    warn!("Fatal controller error; attempting reset");
                    free_range(
                        asqaddr,
                        if self.cap.read().get_bits(0..16) > 4095 {
                            0x3FFC0
                        } else {
                            self.cap.read().get_bits(0..16) - 1
                        },
                    );
                    free_range(
                        acqaddr,
                        if self.cap.read().get_bits(0..16) > 4095 {
                            0xFFF0
                        } else {
                            self.cap.read().get_bits(0..16) - 1
                        },
                    );
                    nvme_error_count += 1;
                    continue 'nvme_init;
                }
                unsafe {
                    halt();
                }
            }
        }
        info!("Controller enabled");
        if self.intmc.read() != 0 {
            info!("Unmasking all interrupts");
            self.intmc.write(0);
        }
        info!("Registering interrupt handler");
        register_interrupt_handler(self.intline, || {
            INTR.store(true, Ordering::SeqCst);
        });
        info!("Sending identify command");
        {
        debug!("Locking SQS");
            let mut sqs = SQS.write();
            debug!("Creating default entry");
            let mut entry = queues::SubmissionQueueEntry::default();
            entry.cdw0.set_bits(0..8, AdminCommand::Identify as u32);
            debug!("Setting CDW0[7:0] to {:X}", AdminCommand::Identify as u32);
            entry.cdw0.set_bits(8..10, 0);
            entry.cdw0.set_bits(10..14, 0);
            entry.cdw0.set_bits(14..16, 0);
            entry.cdw0.set_bits(16..32, 0);
            debug!("Setting CDW0[8:32] to 0");
            entry.nsid = 0;
            entry.prps[0] = self.resp_buffer_addr;
            entry.operands[0].set_bits(16..31, 0);
            entry.operands[0].set_bits(0..8, 1);
            sqs[0].queue_command(entry);
            info!("Identify command sent, awaiting response");
            loop {
                if INTR.load(Ordering::SeqCst) {
                    info!("Identify command returned data");
                    break;
                }
                unsafe {
                    halt();
                }
            }
            INTR.store(false, Ordering::SeqCst);
            // Read data structure
            let mut data = [0u8; 4096];
                for i in 0..4096 {
                    data[i] = self.resp_buffer.index(i).read();
                }
                for i in 0..4096 {
                    self.resp_buffer.index(i).write(0);
                }
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
    }
}
