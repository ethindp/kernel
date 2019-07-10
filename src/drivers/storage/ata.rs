#[repr(u16)]
enum DeviceType {
    Primary = 0x1F0,
    Secondary = 0x170,
    Master = 0x00,
    Slave = 0x01,
}
