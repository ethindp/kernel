use core::mem::size_of;

pub fn log2(n: u64) -> u64 {
    (8 * size_of::<u64>() - (n.leading_zeros() as usize) - 1) as u64
}
