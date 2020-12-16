use bit_field::BitField;
use crossbeam_queue::SegQueue;
use log::*;
use static_assertions::assert_eq_size;
use voladdress::DynamicVolBlock;


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

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
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
    pub(crate) status: u16,
}
// It is critical that this be 16 bytes.
assert_eq_size!(CompletionQueueEntry, [u8; 16]);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct SubmissionQueue {
    addr: usize,
    sqh: u16,
    entries: u16,
}

impl SubmissionQueue {
    pub(crate) fn new(addr: u64, entries: u16) -> Self {
        SubmissionQueue {
            addr: addr as usize,
            sqh: u16::MAX,
            entries,
        }
    }

    pub(crate) fn queue_command(&mut self, entry: SubmissionQueueEntry) {
        let addr: DynamicVolBlock<u32> =
            unsafe { DynamicVolBlock::new(self.addr, (self.entries * 16) as usize) };
        debug!("Current SQH: {}", self.sqh);
        self.sqh = (self.sqh + 1) % self.entries;
        debug!("New SQH: {}", self.sqh);
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
        debug!("Entry data: CDW0 = {:X}, NSID = {:X}, MPTR = ({:X}, {:X}), PRP list = ({:X}, {:X}, {:X}, {:X}), ARGS = ({:X}, {:X}, {:X}, {:X}, {:X}, {:X})", cmd[0], cmd[1], cmd[4], cmd[5], cmd[6], cmd[7], cmd[8], cmd[9], cmd[10], cmd[11], cmd[12], cmd[13], cmd[14], cmd[15]);
        cmd.iter().enumerate().for_each(|(i, c)| {
            debug!(
                "Writing dword {:X}, offset {:X}",
                i,
                (self.sqh as usize) + i
            );
            addr.index((self.sqh as usize) + i).write(*c);
        });
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct CompletionQueue {
    addr: usize,
    cqh: u16,
    entries: u16,
}

impl CompletionQueue {
    pub(crate) fn new(addr: u64, entries: u16) -> Self {
        CompletionQueue {
            addr: addr as usize,
            cqh: u16::MAX,
            entries,
        }
    }

    pub(crate) fn check_queue_for_new_entries(
        &mut self,
        entry_storage_queue: &mut SegQueue<CompletionQueueEntry>
    ) {
        let addr: DynamicVolBlock<u128> =
            unsafe { DynamicVolBlock::new(self.addr, self.entries as usize) };
            self.cqh = (self.cqh + 1) % self.entries;
            // Just consume everything damnit
        (0 .. self.entries as usize).for_each(|i| {
            let entry = addr.index((self.cqh as usize) + i).read();
                let cqe = CompletionQueueEntry {
                    cmdret: entry.get_bits(0..32) as u32,
                    _rsvd: 0,
                    sqhd: entry.get_bits(64..80) as u16,
                    sqid: entry.get_bits(80..96) as u16,
                    cid: entry.get_bits(96..112) as u16,
                    phase: entry.get_bit(112),
                    status: entry.get_bits(113..128) as u16,
                };
                entry_storage_queue.push(cqe);
        });
    }
}
