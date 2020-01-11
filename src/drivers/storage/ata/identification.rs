use alloc::string::String;

// Word 0
#[derive(Clone, Copy)]
pub struct GeneralConfiguration {
pub is_ata: bool, // bit 15
pub data_incomplete: bool, // bit 2
}

// Words 49-50
#[derive(Clone, Copy)]
pub struct Capabilities {
// word 49
pub standard_standby_timer: bool, // bit 15
pub iordy_supported: bool, // bit 11
pub iordy_adjustable: bool, // bit 10
pub lba_supported: bool, // bit 9
pub dma_supported: bool, // bit 8
// Word 50
pub min_standby_vendor: bool, // bit 0
}

// Word 63
#[derive(Clone, Copy)]
pub union MwDmaModesSelected {
pub dma2: bool, // bit 10
pub dma1: bool, // bit 9
pub dma0: bool, // bit 8
}

#[derive(Clone, Copy)]
pub union CurrentNegotiatedSpeed {
pub gen3: bool,
pub gen2: bool,
pub gen1: bool,
}

// Words 76-78
#[derive(Clone, Copy)]
pub struct SataCapabilities {
// word 76
// Bits 15:11 and 7:3 are reserved for SATA.
// Information on these bits was taken from SATA 3.4.
pub read_dma_log_ext_eq_read_log_ext: bool, //bit 15, copy of the READ LOG DMA EXT AS EQUIVALENT TO READ LOG EXT SUPPORTED bit
pub device_apts: bool, //bit 14, copy of the DEVICE AUTOMATIC PARTIAL TO SLUMBER TRANSITIONS SUPPORTED bit
pub host_auto_ptst: bool, // bit 13, copy of the HOST AUTOMATIC PARTIAL TO SLUMBER TRANSITIONS SUPPORTED bit
pub ncq_priority_info: bool, // bit 12, copy of the NCQ PRIORITY INFORMATION SUPPORTED bit
pub unload_while_ncq_outstanding: bool, // bit 11, copy of the UNLOAD WHILE NCQ COMMANDS ARE OUTSTANDING SUPPORTED bit
pub sata_phy: bool, // bit 10
pub partial_slumber_pm: bool, // bit 9
pub ncq: bool, // bit 8
pub gen3: bool, // bit 3, copy of the SATA GEN 3 SIGNALING SPEED SUPPORTED bit
pub gen2: bool, // bit 2, copy of the SATA GEN 2 SIGNALING SPEED SUPPORTED bit
pub gen1: bool, // bit 1, copy of the SATA GEN 1 SIGNALING SPEED SUPPORTED bit
// Word 77
pub oob_management: bool, // bit 9, copy of the OUT OF BAND MANAGEMENT INTERFACE SUPPORTED bit
pub power_disable_always_enabled: bool, // bit 8, copy of the POWER DISABLE FEATURE ALWAYS ENABLED bit
pub devsleep_to_reducedpwrstate: bool, // bit 7, copy of the DEVSLEEP _ TO _ REDUCEDPWRSTATE CAPABILITY SUPPORTED bit
pub snd_recv_queued_cmds: bool, // bit 6, copy of the SEND AND RECEIVE QUEUED COMMANDS SUPPORTED bit
pub ncq_nondata: bool, // bit 5, copy of the NCQ NON - DATA COMMAND SUPPORTED bit
pub ncq_streaming: bool, // bit 4, copy of the NCQ STREAMING SUPPORTED bit
pub negotiated_speed: CurrentNegotiatedSpeed, // bits 3:1, copy of the CURRENT NEGOTIATED SERIAL ATA SIGNAL SPEED field
// Word 78
pub power_disable: bool, // bit 12, copy of the POWER DISABLE FEATURE SUPPORTED bit
pub rebuild_assist: bool, // bit 11, copy of the REBUILD ASSIST SUPPORTED bit
pub dipm_ssp: bool, // bit 10, copy of the DIPM SSP PRESERVATION SUPPORTED bit
pub hybrid_information: bool, // bit 9, copy of the HYBRID INFORMATION SUPPORTED bit
pub device_sleep: bool, // bit 8, copy of the DEVICE SLEEP SUPPORTED bit
pub ncq_autosense: bool, // bit 7, copy of the NCQ AUTOSENSE SUPPORTED bit
pub ssp: bool, // bit 6, copy of the SOFTWARE SETTINGS PRESERVATION SUPPORTED bit

}

