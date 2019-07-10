use crate::memory::*;

#[repr(u16)]
#[derive(Eq, PartialEq)]
pub enum HDARegister {
GlobalCapabilities = 0x00,
MinorVersion = 0x02,
MajorVersion = 0x03,
OutputPayloadCapabilities = 0x04,
InputPayloadCapabilities = 0x06,
GlobalControl = 0x08,
WakeEnable = 0x0C,
StateChangeStatus = 0x0A,
GlobalStatus = 0x10,
OutputStreamPayloadCapability = 0x18,
InputStreamPayloadCapability = 0x1A,
InterruptControl = 0x20,
InterruptStatus = 0x24,
WallClockCounter = 0x30,
StreamSynchronisation = 0x34,
CorbLower = 0x40,
CorbUpper = 0x44,
CorbWrite = 0x48,
CorbRead = 0x4A,
CorbControl = 0x4C,
CorbStatus = 0x4D,
CorbSize = 0x4E,
RirbLower = 0x50,
RirbUpper = 0x54,
RirbWrite = 0x58,
ResponseInterruptCount = 0x5A,
RirbControl = 0x5C,
RirbStatus = 0x5D,
RirbSize = 0x5E,
DmaPosLower = 0x70,
DmaPosUpper = 0x74,
}

pub fn init() {
allocate_phys_range(0xFEBF0000, 0xFEBF0100);
}
