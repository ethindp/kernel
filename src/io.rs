#[repr(u8)]
#[non_exhaustive]
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug)]
pub enum Error {
NotFound(String<U256>),
PermissionDenied(String<U256>),
ConnectionRefused(String<U256>),
ConnectionReset(String<U256>),
ConnectionAborted(String<U256>),
NotConnected(String<U256>),
AddrInUse(String<U256>),
AddrNotAvailable(String<U256>),
BrokenPipe(String<U256>),
AlreadyExists(String<U256>),
WouldBlock(String<U256>),
InvalidInput(String<U256>),
InvalidData(String<U256>),
TimedOut(String<U256>),
WriteZero(String<U256>),
Interrupted(String<U256>),
UnexpectedEof(String<U256>),
Other(String<U256>)
}


#[repr(u8)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug)]
pub enum SeekFrom {
Start(u128),
End(i128),
Current(i128)
}

pub trait Read {
