/// The bus module contains drivers for buses; i.e. USB support.
pub mod bus;
/// The fs module contains drivers for accessing file systems either in RAM or on-disk
pub mod fs;
/// The hid module contains drivers capable of interacting with HIDs.
pub mod hid;
/// The net module contains drivers used to interact with networking peripherals.
pub mod net;
/// The sound module contains modules used to interact with devices such as sound cards.
pub mod sound;
/// The storage module contains drivers used to interact with storage controllers.
pub mod storage;
/// The video module contains drivers for video controls.
pub mod video;
/// The virtio module contains drivers for VirtIO usage.
pub mod virtio;
