use voladdress::DynamicVolBlock;
use bit_field::BitField;
use static_assertions::assert_eq_size;
use heapless::{spsc::Queue, consts::*};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct SubmissionQueueEntry {
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
pub struct CompletionQueueEntry {
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
pub struct SubmissionQueue {
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
pub struct CompletionQueue {
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
