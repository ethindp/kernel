use bit_field::BitField;
use minivec::MiniVec;
use static_assertions::assert_eq_size;
use voladdress::DynamicVolBlock;
use heapless::String;
use heapless::consts::*;
use alloc::format;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct SubmissionQueueEntry {
    /// Command Dword 0, common to all commands
    pub(crate) cdw0: u32,
    /// Namespace ID; FFFFFFFFh refers to all namespaces.
    /// Clear to 0h if this value is unused.
    pub(crate) nsid: u32,
    _rsvd: u64,
    /// Metadata pointer; only used in NVMe over PCIe and if metadata is not interleaved with
    /// logical block data. Rules:
    ///
    /// * If cdw0[PSDT] = 00b, then MPTR shall point to a contiguous physical buffer of
    /// metadata that is dword aligned.
    /// * If cdw0[PSDT] = 01b, then MPTR shall point to a contiguous physical buffer of
    /// metadata that is aligned on any byte boundary.
    /// * If cdw0[PSDT] = 10b, then MPTR shall point to an SGL segment that contains exactly
    /// one SGL descriptor. The descriptor shall be qword aligned and shall:
    ///     * be the first SGL descriptor for the command; or
    ///     * shall be an SGL data block descriptor that shall hold all of the metadata for
    ///     the metadata data transfer and there shall be only one data block descriptor.
    ///
    /// Warning: the controller is not obligated to verify alignment requirements. Therefore,
    /// alignment is automatically guaranteed by the queue entry submission code unless
    /// CDW0[PSDT] = 01b.
    pub(crate) mptr: u64,
    /// Data Pointer (DPTR): This field specifies the data used in the command.
    ///
    /// If CDW0[PSDT] = 00b, then this is a list of PRPs. As noted from the NVMe
    /// specification, PRP Entry 2:
    ///
    /// * is reserved if the data transfer does not cross a memory page boundary;
    /// * specifies the Page Base Address of the second memory page if the data transfer
    /// crosses exactly one memory page boundary, e.g.:
    ///     * the command data transfer length is equal in size to one memory page and the
    ///    offset portion of the PBAO field of PRP1 is non-zero; or
    ///     * the Offset portion of the PBAO field of PRP1 is equal to 0h and the command
    ///     data transfer length is greater than one memory page and less than or equal to
    ///     two memory pages in size; and
    /// * is a PRP List pointer if the data transfer crosses more than one memory page
    /// boundary, e.g.:
    ///     * the command data transfer length is greater than or equal to two memory pages
    ///     in size but the offset portion of the PBAO field of PRP1 is non-zero; or
    ///     * the command data transfer length is equal in size to more than two memory pages
    ///     and the Offset portion of the PBAO field of PRP1 is equal to 0h.
    ///
    /// PRP1 is either a PRP entry or a PRP list pointer depending on the command.
    ///
    /// If CDW0[PSDT] = 01b or 10b, then this is the first SGL segment. If this SGL segment
    /// is an SGL data block, keyed SGL data block, or transport SGL data block descriptor,
    /// then this shall be the only SGL descriptor and shall describe the entire data
    /// transfer. If more than one SGL segment is required, then the firstSGL segment is a
    /// segment or last segment descriptor. See section 4.4 of the NVMe base specification
    /// for more information.
    pub(crate) prps: [u64; 2],
    /// These are command-specific dwords. If this command is a vendor-specific command, then
    /// the vendor-specific command in question may support the NDT and NdM fields. In such
    /// an instance, command dwords 10 and 11 shall be the number of dwords in the data
    /// transfer and number of dwords in the metadata transfer, respectively, with command
    /// dwords 12-15 having command-specific meanings.
    pub(crate) operands: [u32; 6],
}
// It is important that this be exactly 64 bytes.
// Fail compilation if this is not so.
assert_eq_size!(SubmissionQueueEntry, [u8; 64]);

impl SubmissionQueueEntry {
pub(crate) fn new(opcode: u8, optype: OpType, psdt: OpTransferType, cid: u16, nsid: u32, mptr: Option<u64>, dptr: [Option<u64>; 2], operands: [Option<u32>; 6]) -> Self {
let mut entry = Self::default();
entry.cdw0.set_bits(0..8, opcode as u32); // Opcode
entry.cdw0.set_bits(8..10, optype as u32); // Fused operation
entry.cdw0.set_bits(10..14, 0); // Reserved
entry.cdw0.set_bits(14..16, psdt as u32); // PRP or SGL for Data Transfer (PSDT)
entry.cdw0.set_bits(16..32, cid as u32); // Command Identifier (CID)
entry.nsid = nsid;
entry.mptr = if let Some (ptr) = mptr { ptr } else { 0 };
entry.prps[0] = if let Some(prp) = dptr[0] {
ptr
} else {
0
};
entry.prps[1] = if let Some(prp) = dptr[1] {
ptr
} else {
0
};
(0 .. 6).for_each(|idx| if let Some(arg) = operands[idx] {
entry.operands[idx] = arg;
} else {
entry.operands[idx] =0;
});
entry
}
}


#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct CompletionQueueEntry {
    /// Command-specific return value
    pub(crate) cmdret: u32,
    _rsvd: u16,
    /// SQ Head Pointer (SQHD): Indicates the current Submission Queue Head pointer for the
    /// Submission Queue indicated in the SQ Identifier field. This is used to indicate to the host the
    /// Submission Queue entries that have been consumed and may be re-used for new entries.
    /// Note: The value returned is the value of the SQ Head pointer when the completion queue entry
    /// was created. By the time host software consumes the completion queue entry, the controller may
    /// have an SQ Head pointer that has advanced beyond the value indicated.
    pub(crate) sqhd: u16,
    /// SQ Identifier (SQID): Indicates the Submission Queue to which the associated command was
    /// issued. This field is used by host software when more than one Submission Queue shares a single
    /// Completion Queue to uniquely determine the command completed in combination with the
    /// Command Identifier (CID).
    /// This is a reserved field in NVMe over Fabrics implementations.
    pub(crate) sqid: u16,
    /// Command Identifier (CID): Indicates the identifier of the command that is being completed. This
    /// identifier is assigned by host software when the command is submitted to the Submission Queue.
    /// The combination of the SQ Identifier and Command Identifier uniquely identifies the command that
    /// is being completed. The maximum number of requests outstanding for a Submission Queue at
    /// one time is 64 Ki.
    pub(crate) cid: u16,
    /// Phase Tag (P): Identifies whether a Completion Queue entry is new. The Phase Tag values for
    /// all Completion Queue entries shall be initialized to '0' by host software prior to setting CC.EN to
    /// '1'. When the controller places an entry in the Completion Queue, the controller shall invert the
    /// Phase Tag to enable host software to discriminate a new entry. Specifically, for the first set of
    /// completion queue entries after CC.EN is set to '1' all Phase Tags are set to '1' when they are
    /// posted. For the second set of completion queue entries, when the controller has wrapped around
    /// to the top of the Completion Queue, all Phase Tags are cleared to '0' when they are posted. The
    /// value of the Phase Tag is inverted each pass through the Completion Queue.
    /// This is a reserved bit in NVMe over Fabrics implementations.
    pub(crate) phase: bool,
    /// Status Field (SF): Indicates status for the command that is being completed.
    pub(crate) status: Status,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct SubmissionQueue {
    addr: usize,
    qtail: u16,
    entries: u16,
    cid: u16,
}

impl SubmissionQueue {
    pub(crate) fn new(addr: u64, entries: u16) -> Self {
        SubmissionQueue {
            addr: addr as usize,
            qtail: 0,
            entries,
            cid: 0,
        }
    }

    pub(crate) fn queue_command(&mut self, entry: SubmissionQueueEntry) {
        let addr: DynamicVolBlock<u32> =
            unsafe { DynamicVolBlock::new(self.addr, (self.entries * 16) as usize) };
            self.cid = self.cid.wrapping_add(1);
            entry.cdw0.set_bits(16 .. 32, self.cid as u32);
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
        cmd.iter().enumerate().for_each(|(i, c)| {
            addr.index((self.qtail as usize) + i).write(*c);
        });
        self.qtail = self.qtail.wrapping_add(1) % self.entries;
    }

    pub(crate) fn get_queue_tail(&self) -> u16 {
        self.qtail
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct CompletionQueue {
    addr: usize,
    qhead: u16,
    entries: u16,
    phase: bool,
}

impl CompletionQueue {
    pub(crate) fn new(addr: u64, entries: u16) -> Self {
        CompletionQueue {
            addr: addr as usize,
            qhead: 0,
            entries,
            phase: true,
        }
    }

    pub(crate) fn read_new_entries(
        &mut self,
        entry_storage_queue: &mut MiniVec<CompletionQueueEntry>,
    ) {
        let addr: DynamicVolBlock<u128> =
            unsafe { DynamicVolBlock::new(self.addr, self.entries as usize) };
        // Just consume everything damnit
        (0..(self.entries / 16) as usize).for_each(|i| {
            let entry = addr.index(i).read();
            if entry.get_bit(112) == self.phase {
                            self.qhead = self.qhead.wrapping_add(1) % self.entries;
                let cqe = CompletionQueueEntry {
                    cmdret: entry.get_bits(0..32) as u32,
                    _rsvd: 0,
                    sqhd: entry.get_bits(64..80) as u16,
                    sqid: entry.get_bits(80..96) as u16,
                    cid: entry.get_bits(96..112) as u16,
                    phase: entry.get_bit(112),
                    status: Status {
                    dnr: entry.get_bit(127),
                    more: entry.get_bit(126),
                    crd: match entry.get_bits(124 .. 126) {
                    0x00 => CRDType::Immediate,
                    0x01 => CRDType::CRDT1,
                    0x02 => CRDType::CRDT2,
                    0x03 => CRDType::CRDT3,
                    e => CRDType::Other(e as u8)
                    },
                    sct: match entry.get_bits(121 .. 124) {
                    0x00 => StatusCodeType::Generic,
                    0x01 => StatusCodeType::CommandSpecific,
                    0x02 => StatusCodeType::MediaAndDataIntegrity,
                    0x03 => StatusCodeType::Path,
                    0x07 => StatusCodeType::VendorSpecific,
                    e => StatusCodeType::Other(e as u8)
                    },
                    sc: entry.get_bits(113 .. 121) as u8
                    },
                };
                entry_storage_queue.push(cqe);
            }
        });
        self.phase = !self.phase;
    }

    pub(crate) fn get_queue_head(&self) -> u16 {
        self.qhead
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Status {
/// Do Not Retry (DNR): If set to '1', indicates that if the same command is re-submitted to any
/// controller in the NVM subsystem, then that re-submitted command is expected to fail. If cleared to
/// '0', indicates that the same command may succeed if retried.
pub dnr: bool,
/// More (M): If set to '1', there is more status information for this command as part of the Error
/// Information log that may be retrieved with the Get Log Page command. If cleared to '0', there is
/// no additional status information for this command.
pub more: bool,
/// Command Retry Delay
pub crd: CRDType,
/// Status code type
pub sct: StatusCodeType,
/// Status Code
pub sc: u8,
}

impl Status {
/// Translates an error code into an error message
pub fn to_string(&self) -> String<U128> {
let mut msg: String<u128> = String::new();
msg.push_str(match self.sct {
StatusCodeType::Generic => match self.sc {
0x00 => "Successful completion",
0x01 => "Invalid command opcode",
0x02 => "Invalid field in command",
0x03 => "Command ID conflict",
0x04 => "Data transfer error",
0x05 => "Commands aborted due to power loss notification",
0x06 => "Internal error",
0x07 => "Command abort requested",
0x08 => "Command aborted due to SQ deletion",
0x09 => "Command aborted due to failed fused command",
0x0a => "Command aborted due to missing fused command",
0x0b => "Invalid namespace or format",
0x0c => "Command sequence error",
0x0d => "Invalid SGL segment descriptor",
0x0e => "Invalid number of SGL descriptors",
0x0f => "Data SGL length invalid",
0x10 => "Metadata SGL length invalid",
0x11 => "SGL descriptor type invalid",
0x12 => "Invalid use of controller memory buffer",
0x13 => "PRP offset invalid",
0x14 => "Atomic write unit exceeded",
0x15 => "Operation denied",
0x16 => "SGL offset invalid",
0x18 => "Host identifier inconsistent format",
0x19 => "Keep alive timer expired",
0x1a => "Keep alive timer invalid",
0x1b => "Command aborted due to preempt and abort",
0x1c => "Sanitize failed",
0x1d => "Sanitize in progress",
0x1e => "SGL data block granularity invalid",
0x1f => "Command not supported for queue in CMB",
0x20 => "Namespace is write protected",
0x21 => "Command interrupted",
0x22 => "Transient transport error",
0x17 | 0x85 ..= 0xbf | 0x23 ..= 0x7f => "Reserved",
0xc0 ..= 0xff => "Vendor specific",
0x80 => "LBA out of range",
0x81 => "Capacity exceeded",
0x82 => "Namespace not ready",
0x83 => "Reservation conflict",
0x84 => "Format in progress",
},
StatusCodeType::CommandSpecific => match self.sc {
0x00 => "Completion queue invalid",
0x01 => "Invalid queue identifier",
0x02 => "Invalid queue size",
0x03 => "Abort command limit exceeded",
0x05 => "Asynchronous event request limit exceeded",
0x06 => "Invalid firmware slot",
0x07 => "Invalid firmware image",
0x08 => "Invalid interrupt vector",
0x09 => "Invalid log page",
0x0a => "Invalid format",
0x0b => "Firmware activation requires conventional reset",
0x0c => "Invalid queue deletion",
0x0d => "Feature identifier not saveable",
0x0e => "Feature not changeable",
0x0f => "Feature not namespace specific",
0x10 => "Firmware activation requires NVM subsystem reset",
0x11 => "Firmware activation requires controller reset",
0x12 => "Firmware activation requires maximum time violation",
0x13 => "Firmware activation prohibited",
0x14 => "Overlapping range",
0x15 => "Namespace insufficient capacity",
0x16 => "Namespace identifier unavailable",
0x18 => "Namespace already attached",
0x19 => "Namespace is private",
0x1a => "Namespace not attached",
0x1b => "Thin provisioning not supported",
0x1c => "Controller list invalid",
0x1d => "Device self-test in progress",
0x1e => "Boot partition write prohibited",
0x1f => "Invalid controller identifier",
0x20 =>"Invalid secondary controller state",
0x21 => "Invalid number of controller resources",
0x22 => "Invalid resource identifier",
0x23 => "Sanitize prohibited while persistent memory region is enabled",
0x24 => "ANA group identifier invalid",
0x25 => "ANA attach failed",
0x04 | 0x18 | 0x26 ..= 0x6f | 0x83 .. 0xbf => "Reserved",
0xc0 ..= 0xff => "Vendor specific",
0x80 => "Conflicting attributes",
0x81 => "Invalid protection information",
0x82 => "Attempted write to read only range",
0x70 .. 0x7f => "Directive specific",
},
StatusCodeType::MediaAndDataIntegrity => match self.sc {
0x00 ..= 0x7f | 0x88 ..= 0xbf => "Reserved",
0x80 => "Write fault",
0x81 => "Unrecovered read error",
0x82 => "End-to-end guard check error",
0x83 => "End-to-end application tag check error",
0x84 => "End-to-end reference tag check error",
0x85 => "Compare failure",
0x86 => "Access denied",
0x87 => "Deallocated or unwritten logical block",
0xc0 ..= 0xff => "Vendor specific",
},
StatusCodeType::Path => match self.sc {
0x00 => "Internal path error",
0x01 => "Asymmetric access persistent loss",
0x02 => "Asymmetric access inaccessible",
0x03 => "Asymmetric access transition",
0x04 ..= 0x5f | 0x61 ..= 0x6f | 0x72 ..= 0x7f => "Reserved",
0x60 => "Controller pathing error",
0x70 => "Host pathing error",
0x71 => "Command aborted by host",
0x80 ..= 0xbf => "I/O specific",
0xc0 ..= 0xff => "Vendor specific"
},
StatusCodeType::VendorSpecific => format!("Vendor specific (0x{:X})", self.sc).as_str(),
StatusCodeType::Other(c) => format!("Other (0x{:X}): 0x{:X}", c, self.sc).as_str()
});
msg
}
}


#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CRDType {
Immediate,
CRDT1,
CRDT2,
CRDT3,
Other(u8)
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum StatusCodeType {
Generic,
CommandSpecific,
MediaAndDataIntegrity,
Path,
VendorSpecific,
Other(u8)
}

