// SPDX-License-Identifier: MPL-2.0
use crate::memory::{allocate_phys_range, free_range, get_rsdp};
use acpi::fadt::Fadt;
use acpi::sdt::Signature;
use acpi::*;
use core::ptr::NonNull;
use log::*;
use spin::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct AcpiMapper;

impl AcpiHandler for AcpiMapper {
    unsafe fn map_physical_region<T>(&self, addr: usize, size: usize) -> PhysicalMapping<Self, T> {
        allocate_phys_range(addr as u64, (addr + size) as u64, true);
        unsafe {
            PhysicalMapping::new(
                addr,
                NonNull::new(addr as *mut T).unwrap(),
                size,
                size,
                *self,
            )
        }
    }

    fn unmap_physical_region<T>(mapping: &PhysicalMapping<Self, T>) {
        free_range(
            mapping.physical_start() as u64,
            (mapping.physical_start() as u64) + (mapping.region_length() as u64),
        );
    }
}

static TABLES: Once<AcpiTables<AcpiMapper>> = Once::new();

/// Initializes the ACPI tables.
#[cold]
pub async fn init() {
    if !TABLES.is_completed() {
        info!("Initializing ACPI tables");
        let tables = TABLES.call_once(|| {
            let h = AcpiMapper::default();
            unsafe { AcpiTables::from_rsdp(h, get_rsdp() as usize) }.unwrap()
        });
        unsafe {
            if matches!(tables.get_sdt::<Fadt>(Signature::FADT), Ok(_))
                && matches!(tables.get_sdt::<Fadt>(Signature::FADT).unwrap(), Some(_))
            {
                info!("Found FADT");
                let fadt = tables.get_sdt::<Fadt>(Signature::FADT).unwrap().unwrap();
                let sci_int = fadt.sci_interrupt;
                info!("SCI int. is {}", sci_int);
                let smi_cmd = fadt.smi_cmd_port;
                info!("SMI command port: {:X}", smi_cmd);
            }
        }
    } else {
        warn!("Got request to reinitialize acpi; ignoring");
    }
}

/// Returns a list of PCI regions.
pub fn get_pci_regions() -> Result<PciConfigRegions, AcpiError> {
    PciConfigRegions::new(TABLES.get().unwrap())
}

/// Returns information about the high precision event timer (HPET)
#[cold]
pub fn get_hpet_info() -> Result<HpetInfo, AcpiError> {
    HpetInfo::new(TABLES.get().unwrap())
}
