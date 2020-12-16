pub mod ffs;
pub mod mfs;
pub mod ufs;

pub const MAX_RAW_IO: u32 = 64 * 1024;
pub const MAX_BSIZE: u32 = 64 * 1024;
pub const DEV_BSHIFT: u16 = 9;
pub const DEV_BSIZE: u16 = 1 << DEV_BSHIFT;
pub const BLK_DEV_IO_SZ: u16 = PAGE_SIZE;
pub const PAGE_SIZE: u16 = 1 << PAGE_SHIFT;
pub const PAGE_SHIFT: u16 = 12;

pub const fn bytes_to_disk_blocks(x: usize) -> usize {
    x >> (DEV_BSHIFT as usize)
}

pub const fn disk_blocks_to_bytes(x: usize) -> usize {
    x << (DEV_BSHIFT as usize)
}

pub const fn pages_to_disk_blocks(x: usize) -> usize {
    x << ((PAGE_SHIFT - DEV_BSHIFT) as usize)
}

pub const fn disk_blocks_to_pages(x: usize) -> usize {
    x >> ((PAGE_SHIFT - DEV_BSHIFT) as usize)
}
