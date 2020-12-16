use zerocopy::FromBytes;

/// This log page is used to describe extended error information for a command that completed with error or
/// report an error that is not specific to a particular command. Extended error information is provided when
/// the More (M) bit is set to `1` in the Status Field for the completion queue entry associated with the command
/// that completed with error or as part of an asynchronous event with an Error status type. This log page is
/// global to the controller.
///
/// This error log may return the last n errors. If host software specifies a data transfer of the size of n error
/// logs, then the error logs for the most recent n errors are returned. The ordering of the entries is based on
/// the time when the error occurred, with the most recent error being returned as the first log entry.
///
/// Each entry in the log page returned is defined in this structure. The log page is a set of 64-byte entries; the
/// maximum number of entries supported is indicated in the ELPE field in the Identify Controller data structure.
///
/// If the log page is full when a new entry is generated, the controller should insert the
/// new entry into the log and discard the oldest entry.
/// The controller should clear this log page by removing all entries on power cycle and Controller Level Reset.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, FromBytes)]
pub struct ErrorInformationEntry {
/// This is a 64-bit incrementing error count, indicating a unique identifier for this error.
/// The error count starts at 1h, is incremented for each unique error log entry, and is retained across
/// power off conditions. A value of 0h indicates an invalid entry; this value is used when there are
/// lost entries or when there are fewer errors than the maximum number of entries the controller
/// supports.
pub error_count: u64,
/// This field indicates the Submission Queue Identifier of the command that
/// the error information is associated with. If the error is not specific to a particular command, then
/// this field shall be set to FFFFh.
pub submission_queue_id: u16,
/// This field indicates the Command Identifier of the command that the error is
/// associated with. If the error is not specific to a particular command, then this field shall be set to
/// FFFFh.
pub command_id: u16,
/// This field indicates the byte and bit of the command parameter that
/// the error is associated with, if applicable. If the parameter spans multiple bytes or bits, then the
/// location indicates the first byte and bit of the parameter.
/// If the error is not specific to a particular command, then this field shall be set to FFFFh.
pub parameter_error_location: u16,
/// This field indicates the first LBA that experienced the error condition, if applicable.
pub lba: u64,
/// This field indicates the NSID of the namespace that the error is associated with, if applicable.
pub namespace: u32,
