/// This file defines the kernel tasking subsystem (KTS) and its functioning and architecture.
/// The KTS is a trampoline for launching processes. A process consists of three major components:
///
/// * The process identifier (PID), a unique identifier that identifies this process to the system and its users.
/// * The thread identifier (TID), a unique identifier that identifies each thread in the process to the system and its users.
/// * The POSIX process identifier (PPID), a 64-bit unique identifier that is used for POSIX-compliant functions. The lower 32-bits of the value contain the PID; the upper 32-bits contain the TID.
///
/// Computation of the PPID follows the formula: PPID = PID << 32 | TID;
/// unpacking this into a PID/TID pare involves the following two formulas:
///
/// * PID = ((PPID & 0xFFFFFFFF00000000) >> 32)
/// * TID = (PPID & 0xFFFFFFFF)
///
/// The kernel makes these available in this module as functions (create_pid() and unpack_ppid()).

const MAX_OPEN_FILES: usize = u16::max_value() as usize;

#[repr(u8)]
#[derive(Eq, PartialEq)]
pub enum ProcessState {
    Alive,
    Zombie,
    Dead,
}

#[repr(C)]
pub struct Process {
    pub name: &'static str,
    pub pid: u32,
    pub tid: u32,
    pub ppid: u64,
    pub rsp: u64,
    pub stack_top: u64,
    pub rip: u64,
    pub cr3: u64,
    pub state: ProcessState,
    pub num_open_files: u16,
    pub open_files: [Option<&'static str>; MAX_OPEN_FILES],
}

#[no_mangle]
pub extern "C" fn create_ppid(pid: u32, tid: u32) -> u64 {
    (pid as u64) << 32 | (tid as u64)
}

#[no_mangle]
pub extern "C" fn unpack_ppid(ppid: u64) -> (u32, u32) {
    (
        (((ppid & 0xFFFF_FFFF_0000_0000) >> 32) as u32),
        ((ppid & 0xFFFF_FFFF) as u32),
    )
}
