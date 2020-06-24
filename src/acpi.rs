use crate::memory::{allocate_phys_range, free_range};
use crate::printkln;
use acpi::*;
use core::ptr::NonNull;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct AcpiMapper;

impl handler::AcpiHandler for AcpiMapper {
    unsafe fn map_physical_region<T>(
        &mut self,
        physical_address: usize,
        _: usize,
    ) -> handler::PhysicalMapping<T> {
        let addr = physical_address & !(4096-1);
        allocate_phys_range(
            addr as u64,
            (addr + 4095) as u64,
        );
        let ptr = if let Some(p) = NonNull::new(addr as *mut T) { p } else { NonNull::new_unchecked(addr as *mut T) };
        handler::PhysicalMapping {
            physical_start: addr,
            virtual_start: ptr,
            region_length: 4095,
            mapped_length: 4095,
        }
    }

    fn unmap_physical_region<T>(&mut self, region: PhysicalMapping<T>) {
        free_range(
            region.physical_start as u64,
            (region.physical_start + region.mapped_length) as u64,
        );
    }
}

pub fn init() -> Option<Acpi> {
    let mut h = AcpiMapper::default();
    printkln!("init: Searching for ACPI tables");
    let table = match unsafe { search_for_rsdp_bios(&mut h) } {
        Ok(a) => a,
        Err(e) => {
            printkln!("init: ACPI table not found: {:?}", e);
            return None;
        }
    };
    printkln!("init: acpi rev. {} found", table.acpi_revision);
    Some(table)
}
