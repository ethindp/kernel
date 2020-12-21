// SPDX-License-Identifier: MPL-2.0
use crate::memory::allocate_phys_range;
use acpi::*;
use core::ptr::NonNull;
use lazy_static::lazy_static;
use log::*;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct AcpiMapper;

impl AcpiHandler for AcpiMapper {
    unsafe fn map_physical_region<T>(&self, addr: usize, size: usize) -> PhysicalMapping<Self, T> {
        debug!(
            "Checking memory address range {:X}-{:X} of size {} for ACPI region",
            addr,
            (addr + size),
            size
        );
        allocate_phys_range(addr as u64, (addr + size) as u64, true);
        PhysicalMapping {
            physical_start: addr,
            virtual_start: NonNull::new(addr as *mut T).unwrap(),
            region_length: size,
            mapped_length: size,
            handler: *self,
        }
    }

    fn unmap_physical_region<T>(&self, _: &PhysicalMapping<Self, T>) {}
}

lazy_static! {
    static ref TABLES: AcpiTables<AcpiMapper> = {
        let h = AcpiMapper::default();
        unsafe { AcpiTables::search_for_rsdp_bios(h) }.unwrap()
    };
}

pub fn get_pci_regions() -> Result<PciConfigRegions, AcpiError> {
    PciConfigRegions::new(&TABLES)
}
