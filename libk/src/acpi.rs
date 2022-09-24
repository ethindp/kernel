// SPDX-License-Identifier: MPL-2.0
use crate::memory::{allocate_phys_range, free_range, get_rsdp};
use acpi::fadt::Fadt;
use acpi::hpet::*;
use acpi::sdt::Signature;
use acpi::*;
use bit_field::BitField;
use core::ptr::NonNull;
use log::*;
use spin::*;
use voladdress::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct AcpiMapper;

impl AcpiHandler for AcpiMapper {
    unsafe fn map_physical_region<T>(&self, addr: usize, size: usize) -> PhysicalMapping<Self, T> {
        debug!("Mapping {:X}, size {}", addr, size);
        if addr.get_bits(48..64) != 0 {
            panic!(
                "Bits 48 .. 64 are {:X} and must be cleared",
                addr.get_bits(48..64)
            );
        }
        allocate_phys_range(addr as u64, (addr + size) as u64, true, None);
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
        if let Ok(hpet_info) = get_hpet_info() {
            let _ = allocate_phys_range(
                hpet_info.base_address as u64,
                (hpet_info.base_address as u64) + 0x3FF,
                true,
                None,
            );
            let hpet_cfg: VolAddress<u64, Safe, Safe> =
                unsafe { VolAddress::new(hpet_info.base_address) };
            let caps = hpet_cfg.read();
            info!(
                "Found HPET at addr {:X}, rev. id {:X}",
                hpet_info.base_address,
                caps.get_bits(0..8)
            );
            info!("Clock period: {} femptoseconds", caps.get_bits(32..64));
            info!("Vendor ID: {:X}", caps.get_bits(16..32));
            if !caps.get_bit(13) {
                panic!("HPET main counter is not 64 bits wide");
            }
            if (0..caps.get_bits(8..16) as usize)
                .map(|tmr| unsafe {
                    VolAddress::<u64, Safe, Safe>::new(
                        hpet_info.base_address + (0x20 * tmr) + 0x100,
                    )
                })
                .all(|cfg| cfg.read().get_bit(5))
            {
                panic!("Not all HPET timers are 64 bits wide!");
            }
            info!("Enabling HPET");
            let cfg: VolAddress<u64, Safe, Safe> =
                unsafe { VolAddress::new(hpet_info.base_address + 0x10) };
            let mut cur_cfg = cfg.read();
            cur_cfg.set_bit(1, true);
            cur_cfg.set_bit(0, true);
            cfg.write(cur_cfg);
        } else {
            panic!("HPET not supported, but HPET required");
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
pub fn get_hpet_info() -> Result<HpetInfo, AcpiError> {
    HpetInfo::new(TABLES.get().unwrap())
}
