/// The io submodule of the IOF contains general-purpose traits
/// for easy to implement IO support throughout the kernel. It was ported from
// the io package in the go source code.

use core::fmt;

#[derive(Debug, Clone)]
pub struct ShortError;

#[derive(Debug, Clone)]
pub struct ShortBufferError;

#[derive(Debug, Clone)]
pub struct EofError;

#[derive(Debug, Clone)]
pub struct UnexpectedEofError;

#[derive(Debug, Clone)]
pub struct NoProgressError;

impl fmt::Display for ShortError {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
write!(f, "Write accepted smaller byte count than requested");
}
}

impl fmt::Display for ShortBufferError {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
write!(f, "larger buffer required to store all data (incomplete write)");
}
}

impl fmt::Display for EofError {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
write!(f, "No more input available");
}
}

impl fmt::Display for UnexpectedEofError {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
write!(f, "Got unexpected EOF when reading");
}
}

impl fmt::Display for NoProgressError {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
write!(f, "IO: no outputs available (broken IOF?)")
}
}

#[derive(Clone, Copy)]
pub enum SeekLocation {
Start,
Current,
End,
}

// The Reader trait wraps the reading of data.
pub trait Reader {
fn read()->Result<(Vec<u8>, u64), Error>;
}

// The Writer trait wraps writing of data to a particular destination
trait Writer {
fn write(p: Vec<u8>)->Result<u64, Error>;
}

// The Closer trait wraps the closing of readers and writers, as well as any extra deinitialization for the underlying system.
pub trait Closer {
fn close()->Result<(), Error>;
}

// The Seeker trait wraps seeking, if possible, on the underlying system.
// It accepts three seek offsets: start, current and end.
// If start, the seeker must start seeking from the beginning of the stream.
// If current, the seeker must start seeking from the current stream position (if offset == 0) or from the given offset.
// If end, the seeker must start seeking from the end of the stream.
// The seeker returns the offset relative to the start of the file.
pub trait Seeker {
fn seek(offset: u64, location: SeekLocation)->Result<u64, Error>;
}

// Combinations of the above traits

pub trait ReadWriter: Reader, Writer;
pub trait ReadCloser: Reader, Closer;
pub trait WriteCloser: Writer, Closer;
pub trait ReadWriteCloser: Reader, Writer, Closer;
pub trait ReadSeeker: Reader, Seeker;
pub trait WriteSeeker: Writer, Seeker;
pub trait ReadWriteSeeker: Reader, Writer, Seeker;
