// SPDX-License-Identifier: MPL-2.0
use crate::memory::allocate_phys_range;
use acpi::*;
use core::ptr::NonNull;

#[repr(C)]
#[derive(Default)]
struct AcpiMapper;

impl handler::AcpiHandler for AcpiMapper {
    unsafe fn map_physical_region<T>(
        &mut self,
        addr: usize,
        size: usize,
    ) -> handler::PhysicalMapping<T> {
        allocate_phys_range(
            addr as u64,
            (addr + size) as u64,
        );
        handler::PhysicalMapping {
            physical_start: addr,
            virtual_start: NonNull::new(addr as *mut T).unwrap(),
            region_length: size,
            mapped_length: size,
        }
    }

    fn unmap_physical_region<T>(&mut self, _region: PhysicalMapping<T>) {
    /*
        free_range(
            region.physical_start as u64,
            (region.physical_start + region.mapped_length) as u64,
        );
        */
    }
}

pub fn init() -> Result<Acpi, AcpiError> {
    let mut h = AcpiMapper::default();
    unsafe { search_for_rsdp_bios(&mut h) }
}
