pub struct ChaChaContext {
pub input: [u32; 16],
pub pool: [u32; 16],
pub idx: usize,
}

pub struct Poly1305Context {
pub r: [u32; 4],
pub h: [u32; 5],
pub c: [u32; 5],
pub pad: [u32; 4],
pub cidx: usize,
}

pub struct LockContext {
pub chacha: &'static ChaChaContext,
pub poly: &'static Poly1305Context,
pub ad_size: u64,
pub message_size: u64,
pub ad_phase: i32,
}

pub struct 