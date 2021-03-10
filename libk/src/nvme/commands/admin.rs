use super::super::queues::*;
use super::super::structs::*;
use super::super::{NvmeController, Request};
use crate::{
    disk::*,
    memory::{allocate_phys_range, free_range, get_aligned_free_addr},
};
use bit_field::BitField;
use core::{convert::TryInto, mem::size_of};
use log::*;
use minivec::MiniVec;
use voladdress::VolBlock;

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

impl NvmeController {
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
    pub async fn abort(&mut self, cid: u16, sqid: u16) -> Result<bool, Status> {
        match self.process_command(Request {
            qid: 0,
            entry: SubmissionQueueEntry::new(
                AdminCommand::Abort as u8,
                OpType::Independent,
                OpTransferType::Prps,
                0,
                None,
                [None, None],
                [
                    Some(
                        *0_u32
                            .set_bits(0..16, sqid as u32)
                            .set_bits(16..32, cid as u32),
                    ),
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
            ),
        }) {
            Ok(s) => Ok(s.entry.cmdret.get_bit(0)),
            Err(e) => Err(e),
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
    pub async fn async_event_request(&mut self) -> Result<u32, Status> {
        match self.process_command(Request {
            qid: 0,
            entry: SubmissionQueueEntry::new(
                AdminCommand::AsynchronousEventRequest as u8,
                OpType::Independent,
                OpTransferType::Prps,
                0,
                None,
                [None, None],
                [None, None, None, None, None, None],
            ),
        }) {
            Ok(s) => Ok(s.entry.cmdret),
            Err(e) => Err(e),
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
    pub async fn create_io_completion_queue(
        &mut self,
        prp1: Option<u64>,
        qsize: u16,
        qid: u16,
        iv: u16,
        ien: bool,
        pc: bool,
    ) -> Result<(), Status> {
        let prp = if let Some(prp) = prp1 {
            prp
        } else {
            get_aligned_free_addr(
                (size_of::<CompletionQueueEntry>() as u64) * (qsize as u64),
                4096,
            )
        };
        match self.process_command(Request {
            qid: 0,
            entry: SubmissionQueueEntry::new(
                AdminCommand::CreateIoSubmissionQueue as u8,
                OpType::Independent,
                OpTransferType::Prps,
                0,
                None,
                [Some(prp), None],
                [
                    Some(
                        *0u32
                            .set_bits(16..32, qsize as u32)
                            .set_bits(0..16, qid as u32),
                    ),
                    Some(
                        *0u32
                            .set_bits(16..32, iv as u32)
                            .set_bit(1, ien)
                            .set_bit(0, pc),
                    ),
                    None,
                    None,
                    None,
                    None,
                ],
            ),
        }) {
            Ok(_) => {
                self.cqs.push(CompletionQueue::new(prp, qsize));
                info!(
                    "Controller {}: created IO completion queue with PrP {:X}, size {}",
                    self.id, prp, qsize
                );
                Ok(())
            }
            Err(e) => Err(e),
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
    /// is automatically generated for you.
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
    pub async fn create_io_submission_queue(
        &mut self,
        qsize: u16,
        qid: u16,
        cqid: u16,
        qprio: QueuePriority,
        pc: bool,
        nvmsetid: u16,
    ) -> Result<(), Status> {
        let prp = get_aligned_free_addr(
            (size_of::<SubmissionQueueEntry>() as u64) * (qsize as u64),
            4096,
        );
        match self.process_command(Request {
            qid: 0,
            entry: SubmissionQueueEntry::new(
                AdminCommand::CreateIoSubmissionQueue as u8,
                OpType::Independent,
                OpTransferType::Prps,
                0,
                None,
                [Some(prp), None],
                [
                    Some(
                        *0_u32
                            .set_bits(16..32, qsize as u32)
                            .set_bits(0..16, qid as u32),
                    ),
                    Some(
                        *0_u32
                            .set_bits(16..32, cqid as u32)
                            .set_bits(1..3, qprio as u32)
                            .set_bit(0, pc),
                    ),
                    Some(*0_u32.set_bits(0..16, nvmsetid as u32)),
                    None,
                    None,
                    None,
                ],
            ),
        }) {
            Ok(_) => {
                self.sqs.push(SubmissionQueue::new(prp, qsize));
                info!(
                    "Controller {}: created IO submission queue with PRP {:X} and size {}",
                    self.id, prp, qsize
                );
                Ok(())
            }
            Err(e) => Err(e),
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
    pub async fn delete_io_completion_queue(&mut self, qid: u16) -> Result<(), Status> {
        match self.process_command(Request {
            qid: 0,
            entry: SubmissionQueueEntry::new(
                AdminCommand::DeleteIoCompletionQueue as u8,
                OpType::Independent,
                OpTransferType::Prps,
                0,
                None,
                [None, None],
                [Some(qid as u32), None, None, None, None, None],
            ),
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
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
    pub async fn delete_io_submission_queue(&mut self, qid: u16) -> Result<(), Status> {
        match self.process_command(Request {
            qid: 0,
            entry: SubmissionQueueEntry::new(
                AdminCommand::DeleteIoSubmissionQueue as u8,
                OpType::Independent,
                OpTransferType::Prps,
                0,
                None,
                [None, None],
                [Some(qid as u32), None, None, None, None, None],
            ),
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
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
    pub async fn doorbell_buffer_config(&mut self, prp1: u64, prp2: u64) -> Result<(), Status> {
        match self.process_command(Request {
            qid: 0,
            entry: SubmissionQueueEntry::new(
                AdminCommand::DoorbellBufferConfig as u8,
                OpType::Independent,
                OpTransferType::Prps,
                0,
                None,
                [Some(prp1), Some(prp2)],
                [None; 6],
            ),
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
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
    /// range <math xmlns="http://www.w3.org/1998/Math/MathML"><semantics><mrow><mrow>  <mn>1</mn>  <mo>&leq;</mo>  <mrow>    <mrow>      <mrow>        <mi>n</mi>        <mo>&InvisibleTimes;</mo>        <mi>s</mi>      </mrow>      <mo>&InvisibleTimes;</mo>      <mi>i</mi>    </mrow>    <mo>&InvisibleTimes;</mo>    <mi>d</mi>  </mrow></mrow><mo>&leq;</mo><mn>4294967294</mn></mrow><annotation-xml encoding="MathML-Content"><apply>  <leq></leq>  <apply>    <leq></leq>    <cn>1</cn>    <apply>      <times></times>      <ci>n</ci>      <ci>s</ci>      <ci>i</ci>      <ci>d</ci>    </apply>  </apply>  <cn>4294967294</cn></apply></annotation-xml></semantics></math>, the specified namespace shall be included in the
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
    pub async fn device_self_test(&mut self, stc: SelfTestCode, nsid: u32) -> Result<(), Status> {
        match self.process_command(Request {
            qid: 0,
            entry: SubmissionQueueEntry::new(
                AdminCommand::DeviceSelfTest as u8,
                OpType::Independent,
                OpTransferType::Prps,
                nsid,
                None,
                [None; 2],
                [Some(stc as u32), None, None, None, None, None],
            ),
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Identify command, see sec. 5.15 of NVMe base specification, rev. 1.4b
    ///
    /// # Arguments
    ///
    /// * Data pointer (dptr): specifies a PRP for the data returned by this command. Only one
    /// PRP is required and it mustn't cross a page boundary. This is handled automatically.
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
    pub async fn identify(
        &mut self,
        nsid: u32,
        cntid: u16,
        cns: u8,
        nvmsetid: u16,
        uuid: u8,
    ) -> Result<IdentifyResponse, Status> {
        let dptr = {
            let mut ptr = get_aligned_free_addr(4096, 4096);
            loop {
                if !allocate_phys_range(ptr, ptr + 4096, false) {
                    ptr = get_aligned_free_addr(4096, 4096);
                } else {
                    break;
                }
            }
            ptr
        };
        let mut dword10 = 0u32;
        if cns == 0x12 || cns == 0x13 || cns == 0x14 || cns == 0x15 {
            dword10.set_bits(16..32, cntid as u32);
        }
        dword10.set_bits(0..8, cns as u32);
        let mut dword11 = 0u32;
        if cns == 0x04 {
            dword11.set_bits(0..16, nvmsetid as u32);
        }
        let mut dword14 = 0u32;
        if cns == 0x17 {
            dword14.set_bits(0..7, (uuid as u32) % (1 << 6));
        }
        let mut ns = 0u32;
        if cns == 0x00 || cns == 0x02 || cns == 0x03 || cns == 0x10 || cns == 0x11 || cns == 0x12 {
            ns = nsid;
        }
        match self.process_command(Request {
            qid: 0,
            entry: SubmissionQueueEntry::new(
                AdminCommand::Identify as u8,
                OpType::Independent,
                OpTransferType::Prps,
                ns,
                None,
                [Some(dptr), None],
                [
                    Some(dword10),
                    Some(dword11),
                    None,
                    None,
                    Some(dword14),
                    None,
                ],
            ),
        }) {
            Ok(_) => {
                let mut data = [0u8; 4096];
                let addr: VolBlock<u8, 4096> = unsafe { VolBlock::new(dptr as usize) };
                data.iter_mut().enumerate().for_each(|(i, e)| {
                    *e = addr.index(i).read();
                    addr.index(i).write(0);
                });
                free_range(dptr, dptr + 4096);
                // Check CNS.
                // If we get any errors here, it suggests faulty hardware or programming
                let result = match cns {
                    0x00 => {
                        let (head, body, _) =
                            unsafe { data.align_to_mut::<IdentifyNamespaceResponse>() };
                        if !head.is_empty() {
                            error!("Alignment error: ID(dptr => {:X}, cntid => {:X}, cns => {:X}, nvmsetid => {:X}, uuid => {:X}): got {:X} bytes in head with {:X} bytes in body", dptr, cntid, cns, nvmsetid, uuid, head.len(), body.len());
                            Err(Status {
                                dnr: true, // Maybe retrying will solve the problem
                                more: false,
                                crd: CrdType::Immediate,
                                sct: StatusCodeType::Other(0xFF),
                                sc: 0x00,
                            })
                        } else {
                            let mut s = body[0];
                            let nsguid = s.nsguid;
                            s.nsguid = nsguid.to_be();
                            let eui64 = s.eui64;
                            s.eui64 = eui64.to_be();
                            Ok(IdentifyResponse::IdNamespace(s))
                        }
                    }
                    0x01 => {
                        let (head, body, _) =
                            unsafe { data.align_to_mut::<IdentifyControllerResponse>() };
                        if !head.is_empty() {
                            error!("Alignment error: ID(dptr => {:X}, cntid => {:X}, cns => {:X}, nvmsetid => {:X}, uuid => {:X}): got {:X} bytes in head with {:X} bytes in body", dptr, cntid, cns, nvmsetid, uuid, head.len(), body.len());
                            Err(Status {
                                dnr: true, // Maybe retrying will solve the problem
                                more: false,
                                crd: CrdType::Immediate,
                                sct: StatusCodeType::Other(0xFF),
                                sc: 0x00,
                            })
                        } else {
                            let mut s = body[0];
                            let fguid = s.fguid;
                            s.fguid = fguid.to_be();
                            Ok(IdentifyResponse::IdController(s))
                        }
                    }
                    0x02 => {
                        let mut nslist = [0u32; 1024];
                        let mut nsidx = 0usize;
                        while let Some(chunk) = data.chunks_exact(4).next() {
                            nslist[nsidx] = u32::from_le_bytes(chunk.try_into().unwrap());
                            nsidx += 1;
                        }
                        Ok(IdentifyResponse::ActiveNsList(nslist))
                    }
                    0x03 => {
                        let mut nids: MiniVec<NsIdentifierDescriptor> =
                            MiniVec::with_capacity(4096 / size_of::<NsIdentifierDescriptor>());
                        let mut idx = 0usize;
                        while data[idx] != 0x00 || data[idx + 1] != 0x00 {
                            let length = data[idx + 1] as usize;
                            if data[idx] == 0x01 {
                                nids.push(NsIdentifierDescriptor::IeeeOui(u64::from_be_bytes(
                                    data[idx + 4..idx + length + 4].try_into().unwrap(),
                                )));
                            } else if data[idx] == 0x02 {
                                nids.push(NsIdentifierDescriptor::NsGuid(u128::from_be_bytes(
                                    data[idx + 4..idx + length + 4].try_into().unwrap(),
                                )));
                            } else if data[idx] == 0x03 {
                                nids.push(NsIdentifierDescriptor::NsUuid(u128::from_be_bytes(
                                    data[idx + 4..idx + length + 4].try_into().unwrap(),
                                )));
                            } else {
                                idx += length + 4;
                                continue;
                            }
                            idx += length + 4;
                        }
                        Ok(IdentifyResponse::NsDescList(nids))
                    }
                    0x04 => {
                        let (head, body, _) = unsafe { data.align_to_mut::<NvmSetList>() };
                        if !head.is_empty() {
                            error!("Alignment error: ID(dptr => {:X}, cntid => {:X}, cns => {:X}, nvmsetid => {:X}, uuid => {:X}): got {:X} bytes in head with {:X} bytes in body", dptr, cntid, cns, nvmsetid, uuid, head.len(), body.len());
                            Err(Status {
                                dnr: true, // Maybe retrying will solve the problem
                                more: false,
                                crd: CrdType::Immediate,
                                sct: StatusCodeType::Other(0xFF),
                                sc: 0x00,
                            })
                        } else {
                            Ok(IdentifyResponse::NvmSetList(body[0]))
                        }
                    }
                    0x10 => {
                        let mut nslist = [0u32; 1024];
                        let mut nsidx = 0usize;
                        while let Some(chunk) = data.chunks_exact(4).next() {
                            nslist[nsidx] = u32::from_le_bytes(chunk.try_into().unwrap());
                            nsidx += 1;
                        }
                        Ok(IdentifyResponse::AllocNsList(nslist))
                    }
                    0x11 => {
                        let (head, body, _) =
                            unsafe { data.align_to_mut::<IdentifyNamespaceResponse>() };
                        if !head.is_empty() {
                            error!("Alignment error: ID(dptr => {:X}, cntid => {:X}, cns => {:X}, nvmsetid => {:X}, uuid => {:X}): got {:X} bytes in head with {:X} bytes in body", dptr, cntid, cns, nvmsetid, uuid, head.len(), body.len());
                            Err(Status {
                                dnr: true, // Maybe retrying will solve the problem
                                more: false,
                                crd: CrdType::Immediate,
                                sct: StatusCodeType::Other(0xFF),
                                sc: 0x00,
                            })
                        } else {
                            Ok(IdentifyResponse::AllocIdNamespace(body[0]))
                        }
                    }
                    0x12 => {
                        let mut cntrllist = [0u16; 2048];
                        let mut cntrlidx = 0usize;
                        while let Some(chunk) = data.chunks_exact(4).next() {
                            cntrllist[cntrlidx] = u16::from_le_bytes(chunk.try_into().unwrap());
                            cntrlidx += 1;
                        }
                        Ok(IdentifyResponse::NsAttachedControllerList(cntrllist))
                    }
                    0x13 => {
                        let mut cntrllist = [0u16; 2048];
                        let mut cntrlidx = 0usize;
                        while let Some(chunk) = data.chunks_exact(4).next() {
                            cntrllist[cntrlidx] = u16::from_le_bytes(chunk.try_into().unwrap());
                            cntrlidx += 1;
                        }
                        Ok(IdentifyResponse::ControllerList(cntrllist))
                    }
                    0x14 => {
                        let (head, body, _) =
                            unsafe { data.align_to_mut::<PrimaryControllerCapabilities>() };
                        if !head.is_empty() {
                            error!("Alignment error: ID(dptr => {:X}, cntid => {:X}, cns => {:X}, nvmsetid => {:X}, uuid => {:X}): got {:X} bytes in head with {:X} bytes in body", dptr, cntid, cns, nvmsetid, uuid, head.len(), body.len());
                            Err(Status {
                                dnr: true, // Maybe retrying will solve the problem
                                more: false,
                                crd: CrdType::Immediate,
                                sct: StatusCodeType::Other(0xFF),
                                sc: 0x00,
                            })
                        } else {
                            Ok(IdentifyResponse::PrimaryControllerCapabilities(body[0]))
                        }
                    }
                    0x15 => {
                        let (head, body, _) = unsafe { data.align_to_mut::<ScList>() };
                        if !head.is_empty() {
                            error!("Alignment error: ID(dptr => {:X}, cntid => {:X}, cns => {:X}, nvmsetid => {:X}, uuid => {:X}): got {:X} bytes in head with {:X} bytes in body", dptr, cntid, cns, nvmsetid, uuid, head.len(), body.len());
                            Err(Status {
                                dnr: true, // Maybe retrying will solve the problem
                                more: false,
                                crd: CrdType::Immediate,
                                sct: StatusCodeType::Other(0xFF),
                                sc: 0x00,
                            })
                        } else {
                            Ok(IdentifyResponse::ScList(body[0]))
                        }
                    }
                    0x16 => {
                        let (head, body, _) = unsafe { data.align_to_mut::<NsGranularityList>() };
                        if !head.is_empty() {
                            error!("Alignment error: ID(dptr => {:X}, cntid => {:X}, cns => {:X}, nvmsetid => {:X}, uuid => {:X}): got {:X} bytes in head with {:X} bytes in body", dptr, cntid, cns, nvmsetid, uuid, head.len(), body.len());
                            Err(Status {
                                dnr: true, // Maybe retrying will solve the problem
                                more: false,
                                crd: CrdType::Immediate,
                                sct: StatusCodeType::Other(0xFF),
                                sc: 0x00,
                            })
                        } else {
                            Ok(IdentifyResponse::NsGranList(body[0]))
                        }
                    }
                    0x17 => {
                        let (head, body, _) = unsafe { data.align_to_mut::<UuidList>() };
                        if !head.is_empty() {
                            error!("Alignment error: ID(dptr => {:X}, cntid => {:X}, cns => {:X}, nvmsetid => {:X}, uuid => {:X}): got {:X} bytes in head with {:X} bytes in body", dptr, cntid, cns, nvmsetid, uuid, head.len(), body.len());
                            Err(Status {
                                dnr: true, // Maybe retrying will solve the problem
                                more: false,
                                crd: CrdType::Immediate,
                                sct: StatusCodeType::Other(0xFF),
                                sc: 0x00,
                            })
                        } else {
                            Ok(IdentifyResponse::UuidList(body[0]))
                        }
                    }
                    _ => Ok(IdentifyResponse::Other(data)),
                };
                result
            }
            Err(e) => Err(e),
        }
    }
}

/// This enumeration holds all possible return values for the identify command.
#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IdentifyResponse {
    /// Identify Namespace data structure for the specified NSID or the common namespace
    /// capabilities
    IdNamespace(IdentifyNamespaceResponse),
    /// Identify Controller data structure for the controller processing the command
    IdController(IdentifyControllerResponse),
    /// Active Namespace ID list
    ActiveNsList([u32; 1024]),
    /// Namespace Identification Descriptor list
    NsDescList(MiniVec<NsIdentifierDescriptor>),
    /// NVM set list
    NvmSetList(NvmSetList),
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
    ScList(ScList),
    /// Namespace Granularity List
    NsGranList(NsGranularityList),
    /// UUID List
    UuidList(UuidList),
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

/// Self-test Code definitions
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SelfTestCode {
    /// Start a short device self-test operation
    Short = 0x01,
    /// Start an extended device self-test operation
    Extended,
    /// Vendor specific
    VendorSpecific = 0x0E,
    /// Abort device self-test operation
    Abort = 0x0F,
}
