// SPDX-License-Identifier: MPL-2.0
use core::any::Any;
use core::result::Result;

/// The Disk trait provides common operations that any disk controller should have.
/// This ensures that all implementations of disk devices can be used without writing custom handlers for various disk standards.
pub trait Disk {
    type Error;
    /// The flush function causes any unwritten data for this disk to be delivered to the device
    /// for writing to the media immediately, regardless of whether the device contains a non-volatile cache or not.
    fn flush(&self) -> Result<(), Self::Error>;
    /// The identify function returns disk identification information.
    fn identify<T: Any + Clone + Copy + Eq + PartialEq>(&self) -> Result<T, Self::Error>;
}
