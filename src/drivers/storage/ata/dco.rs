/// Device Configuration Identify data structure
/// See 7.10.3.6 of INCITS 452-2009 (R2019) for information on this structure.
#[derive(Clone, Copy)]
pub struct Identification {
pub revision: u16,
pub mw_dma_modes: MwDmaModes,
pub udma_modes: UdmaModes,
pub max_lba: u64,
pub gen_cmd_set: GenCmdSet,
pub sata_cmd_set: SataCmdSet,
}

///  Multiword DMA modes supported (DCO)
#[derive(Clone, Copy)]
pub struct MwDmaModes {
pub dma2: bool,
pub dma1: bool,
pub dma0: bool,
}

/// Ultra DMA modes supported
#[derive(Clone, Copy)]
pub struct UdmaModes {
pub dma6: bool,
pub dma5: bool,
pub dma4: bool,
pub dma3: bool,
pub dma2: bool,
pub dma1: bool,
pub dma0: bool,
}

/// Command/Feature Sets Supported
#[derive(Clone, Copy)]
pub struct  GenCmdSet {
pub write_read_verify: bool,
pub smart_conveyance_self_test: bool,
pub smart_selective_self_test: bool,
pub fua: bool,
pub streaming: bool,
pub lba48: bool,
pub hpa: bool,
pub aam: bool,
pub tcq: bool,
pub puis: bool,
pub security: bool,
pub smart_error_log: bool,
pub smart_self_test: bool,
pub smart: bool,
pub nvcache: bool,
pub nvcache_power_management: bool,
pub write_uncorrectable_ext: bool,
pub trusted_computing: bool,
pub freefall_control: bool,
}

/// Serial ATA Command set/feature set supported
#[derive(Clone, Copy)]
pub struct alterableSataCommandSet {
pub ssp: bool,
pub async_notifications: bool,
pub interface_power_management: bool,
pub nonzero_buffer_offsets: bool,
pub ncq: bool,
}

