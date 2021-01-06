// SPDX-License-Identifier: MPL-2.0
use core::any::Any;
use core::result::Result;
use minivec::MiniVec;

/// The Disk trait provides common operations that any disk controller should have.
/// This ensures that all implementations of disk devices can be used without writing custom handlers for various disk standards.
pub trait Disk {
    /// The error type that this disk implementation shall return upon errors.
    type Error;
    /// The type of request that this disk implementation shall accept.
    type CommandRequest;
    /// The type of response that this disk implementation shall return.
    type Response;
    /// The process_command function submits a command to the hardware or software
    /// implementation.
    fn process_command(req: Self::CommandRequest) -> Result<Self::Response, Self::Error>;
    /// The process_commands function shall process all command requests and shall return all
    /// responses or errors for each processed command. This function is useful for
    /// implementations that allow batch processing.
    fn process_commands(reqs: MiniVec<Self::CommandRequest>) -> MiniVec<Self::Response>;
}
