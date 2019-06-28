use spin::Mutex;
use crate::pci::{get_devices, PCIDevice};
use crate::printkln;
use crate::pcidb;
use bit_field::BitField;

pub fn init() {
printkln!("HDA: scanning for devices");
for device in get_devices().iter() {
match device.device {
0x7a07 | 0x437b | 0x026c | 0x03e4 | 0x03f0 | 0x055c | 0x055d | 0x07fc | 0x0ac0 | 0x0ac1 | 0x0ac2 | 0x0ac3 | 0x0d94 | 0x0fb0 | 0x0fb8 | 0x0fb9 | 0x0fba | 0x0fbb | 0x1c20 | 0x3288 | 0x9141 | 0x9142 | 0x0f04 | 0x1d20 | 0x1e20 | 0x2284 | 0x2668 | 0x269a | 0x27d8 | 0x3b56 | 0x3b57 | 0x8c20 | 0x8c21 | 0x9ca0 | 0x9dc8 => {
printkln!("HDA: Found device {}, vendor {}, class {:X}, subclass {:X}, bus {:X}, function {:X}", pcidb::get_device_string(device.device), pcidb::get_vendor_string(device.vendor), device.class, device.subclass, device.bus, device.func);
configure_hda(*device);
},
_ => ()
}
}
}

fn configure_hda(device: PCIDevice) {
if device.header_type == 0x00 && device.gen_dev_tbl.is_some() {
let tbl = device.gen_dev_tbl.unwrap();
if tbl.bar0.get_bits(1 .. 2) == 0 {
printkln!("HDA: BAR0: {:X}, BAR1: {:X}, BAR2: {:X}, BAR3: {:X}, BAR4: {:X}, BAR5: {:X}", tbl.bar0, tbl.bar1, tbl.bar2, tbl.bar3, tbl.bar4, tbl.bar5);
}
}
}
