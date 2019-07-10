extern crate alloc;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref MOUNT_POINTS: Mutex<Vec<Mountpoint>> = Mutex::new(Vec::new());
}

#[repr(C)]
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum FileType {
    File = 0x1,
    Directory,
    Pipe,
    Tty,
    BlockDevice,
    CharacterDevice,
}

#[repr(C)]
pub struct File {
    pub refs: u64,
    pub ftype: FileType,
    pub data: &'static [u8],
}

#[repr(C)]
pub struct Directory {
    pub name: &'static str,
    pub file: &'static File,
}

pub trait FSDriver {
    #[no_mangle]
    extern "C" fn open(f: &File, flags: u64);
    #[no_mangle]
    extern "C" fn close(f: File);
    #[no_mangle]
    extern "C" fn read(f: &File, nbyte: usize, offset: usize) -> (&[u8], usize);
    #[no_mangle]
    extern "C" fn write(f: File, buffer: &[u8], nbytes: usize, offsize: usize) -> usize;
    #[no_mangle]
    extern "C" fn read_dir(dir: &File, entry: &Directory, offset: u64) -> isize;
}

pub struct Mountpoint {
    pub root: &'static File,
    pub path: &'static str,
}

#[no_mangle]
pub extern "C" fn fs_get(mut f: File) -> File {
    f.refs += 1;
    f
}

#[no_mangle]
pub extern "C" fn fs_put(mut f: File) -> Option<File> {
    if f.refs > 0 {
        f.refs -= 1;
        Some(f)
    } else {
        None
    }
}
