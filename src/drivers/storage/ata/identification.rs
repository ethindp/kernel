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
pub union MwDmaModeSelected {
pub dma2: bool, // bit 10
pub dma1: bool, // bit 9
pub dma0: bool, // bit 8
pub unknown: bool,
}

#[derive(Clone, Copy)]
pub union CurrentNegotiatedSpeed {
pub gen3: bool,
pub gen2: bool,
pub gen1: bool,
pub unknown: bool,
}

// Words 76-78
#[derive(Clone, Copy)]
pub struct SataCapabilities {
// word 76
// Bits 15:11 and 7:3 are reserved for SATA.
// Information on these bits was taken from SATA 3.4.
pub rlde_eq_rle: bool, //bit 15, copy of the READ LOG DMA EXT AS EQUIVALENT TO READ LOG EXT SUPPORTED bit
pub device_aptst: bool, //bit 14, copy of the DEVICE AUTOMATIC PARTIAL TO SLUMBER TRANSITIONS SUPPORTED bit
pub host_aptst: bool, // bit 13, copy of the HOST AUTOMATIC PARTIAL TO SLUMBER TRANSITIONS SUPPORTED bit
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
pub hardware_feature_control: bool, // bit 5, copy of the HARDWARE FEATURE CONTROL SUPPORTED bit
pub in_order_data_delivery: bool, // bit 4, copy of the IN - ORDER DATA DELIVERY SUPPORTED bit
pub dipm: bool, // bit 3, copy of the DEVICE INITIATED POWER MANAGEMENT SUPPORTED bit
pub dma_setup_auto_activation: bool, // bit 2, copy of the DMA SETUP AUTO - ACTIVATION SUPPORTED bit
pub nonzero_buffer_offsets: bool, // bit 1, copy of the NON - ZERO BUFFER OFFSETS SUPPORTED bit
}

// Word 79
#[derive(Clone, Copy)]
pub struct EnabledSataCapabilities {
pub rebuild_assist: bool, // bit 11, copy of the REBUILD ASSIST ENABLED bit
pub power_disable: bool, // bit 10, copy of the POWER DISABLE FEATURE ENABLED bit
pub hybrid_information: bool, // bit 9, copy of the HYBRID INFORMATION ENABLED bit
pub device_sleep: bool, // bit 8, copy of the DEVICE SLEEP ENABLED bit
pub aptst: bool, // bit 7, copy of the AUTOMATIC PARTIAL TO SLUMBER TRANSITIONS ENABLED bit
pub ssp: bool, // bit 6, copy of the SOFTWARE SETTINGS PRESERVATION ENABLED bit
pub hardware_feature_control: bool, // bit 5, copy of the HARDWARE FEATURE CONTROL ENABLED bit
pub in_order_data_delivery: bool, // bit 4, copy of the IN - ORDER DATA DELIVERY ENABLED bit
pub dipm: bool, // bit 3, copy of the DEVICE INITIATED POWER MANAGEMENT ENABLED bit
pub dma_setup_auto_activate: bool, // bit 2, copy of the DMA SETUP FIS AUTO - ACTIVATE ENABLED bit
pub nonzero_buffer_offsets: bool, // bit 1, copy of the NON - ZERO BUFFER OFFSETS ENABLED bit
}

// Words 82-84 and 119
#[derive(Clone, Copy)]
pub struct SupportedCommandFeatureSets {
// word 82
pub nop: bool, // bit 14
pub read_buffer: bool, // bit 13
pub write_buffer: bool, // bit 12
pub hpa: bool, // bit 10
pub device_reset: bool, // bit 9
pub service: bool, // bit 8
pub release: bool, // bit 7
pub read_lookahead: bool, // bit 6
pub volatile_write_cache: bool, //bit 5
pub atapi: bool, // bit 4
pub power_management: bool, // bit 3
pub security: bool, // bit 1
pub smart: bool, // bit 0
// word 83
pub flush_cache_ext: bool, // bit 13
pub flush_cache: bool, // bit 12
pub dco: bool, // bit 11
pub lba48: bool, // bit 10
pub aam: bool, // bit 9
pub hpa_security: bool, // bit 8
pub set_features_req: bool, // bit 6
pub puis: bool, // bit 5
pub apm: bool, // bit 3
pub cfa: bool, // bit 2
pub tcq: bool, // bit 1
pub download_microcode: bool, // bit 0
// word 84
pub idle_immediate_unload: bool, // bit 13
pub wwn: bool, // bit 8
pub write_dma_queued_fua_ext: bool, // bit 7
pub write_multiple_fua_ext: bool, // bit 6
pub gpl: bool, // bit 5
pub streaming: bool, // bit 4
pub media_card_passthrough: bool, // bit 3
pub media_sn: bool, // bit 2
pub smart_self_test: bool, // bit 1
pub smart_error_logging: bool, // bit 0
// word 119
pub freefall_control: bool, // bit 5
pub download_microcode_offset: bool, // bit 4
pub rw_log_dma_ext: bool, // bit 3
pub write_uncorrectable_ext: bool, // bit 2
pub write_read_verify: bool, // bit 1
}

// words 85-87 and 120
#[derive(Clone, Copy)]
pub struct EnabledCommandFeatureSets {
// word 85
pub nop: bool, // bit 14
pub read_buffer: bool, // bit 13
pub write_buffer: bool, // bit 12
pub hpa: bool, // bit 10
pub device_reset: bool, // bit 9
pub service: bool, // bit 8
pub release: bool, // bit 7
pub read_lookahead: bool, // bit 6
pub volatile_write_cache: bool, //bit 5
pub atapi: bool, // bit 4
pub power_management: bool, // bit 3
pub security: bool, // bit 1
pub smart: bool, // bit 0
// word 86
pub flush_cache_ext: bool, // bit 13
pub flush_cache: bool, // bit 12
pub dco: bool, // bit 11
pub lba48: bool, // bit 10
pub aam: bool, // bit 9
pub hpa_security: bool, // bit 8
pub set_features_req: bool, // bit 6
pub puis: bool, // bit 5
pub apm: bool, // bit 3
pub cfa: bool, // bit 2
pub tcq: bool, // bit 1
pub download_microcode: bool, // bit 0
// word 87
pub idle_immediate_unload: bool, // bit 13
pub wwn: bool, // bit 8
pub write_dma_queued_fua_ext: bool, // bit 7
pub write_multiple_fua_ext: bool, // bit 6
pub smart_self_test: bool, // bit 5
pub media_card_passthrough: bool, // bit 3
pub valid_media_sn: bool, // bit 2
pub smart_error_logging: bool, // bit 0
// word 120
pub freefall_control: bool, // bit 5
pub download_microcode_offset: bool, // bit 4
pub rw_log_dma_ext: bool, // bit 3
pub write_uncorrectable_ext: bool, // bit 2
pub write_read_verify: bool, // bit 1
}

// Word 88
#[derive(Clone, Copy)]
pub union SelectedUdmaMode {
// bits 14:8
pub udma6: bool,
pub udma5: bool,
pub udma4: bool,
pub udma3: bool,
pub udma2: bool,
pub udma1: bool,
pub udma0: bool,
pub unknown: bool,
}

// word 88
#[derive(Clone, Copy)]
pub struct SupportedUdmaModes {
// bits 6:0
pub udma6: bool,
pub udma5: bool,
pub udma4: bool,
pub udma3: bool,
pub udma2: bool,
pub udma1: bool,
pub udma0: bool,
}

// Word 128
#[derive(Clone, Copy)]
pub struct SecurityStatus {
pub master_pwd_cap: bool, // bit 8
pub enhanced_erase: bool, // bit 5
pub pwd_counter_exceeded: bool, // bit 4
pub frozen: bool, // bit 3
pub locked: bool, // bit 2
}

// Word 160
#[derive(Clone, Copy)]
pub struct CFAPowerMode {
pub mode1: bool, // bit 13
pub mode0: bool, // bit 12
pub rms_current: u16, // bits 11:0
}

// Word 206
#[derive(Clone, Copy)]
pub struct SCTCommandTransport {
pub sct_data_tables: bool, // bit 5
pub sct_feature_control: bool, // bit 4
pub sct_error_recovery: bool, // bit 3
pub sct_write_same: bool, // bit 2
pub sct_rw_long: bool, // bit 1
pub sct_supported: bool, // bit 0
}

// Word 214
#[derive(Clone, Copy)]
pub struct NVCacheCapabilities {
pub nvcache_enabled: bool, // bit 4
pub nvcache_pm_enabled: bool, // bit 1
pub nvcache_power_supported: bool, // bit 0
}


// Full identify device structure
#[derive(Clone)]
pub struct DeviceIdentification {
pub gen_config: GeneralConfiguration, // word 0
pub spec_config: u16, // word 2
pub sn: String, // words 10-19
pub fw_rev: String, // words 23-26
pub model_number: String, // words 27-46
pub max_sectors_drq_blk: u8, // word 47
pub tc_supported: bool, // word 48
pub capabilities: Capabilities, // words 49-50
pub logical_sectors_rw_multiple: u8, // word 59
pub total_sectors_lba28: u32, // words 60-61
pub selected_mwdma_mode: MwDmaModeSelected, // word 63
pub min_mwdma_cycle_time: u16, // word 65
pub recommended_mwdma_cycle_time: u16, // word 66
pub min_pio_cycle_time_no_iordy: u16, // word 67
pub min_pio_cycle_time_iordy: u16, // word 68
pub queue_depth: u8, // word 75
pub sata_caps: SataCapabilities, // words 76-78
pub en_sata_caps: EnabledSataCapabilities, // word 79
pub major: u16, // word 80
pub minor: u16, // word 81
pub cmd_ft_sets: SupportedCommandFeatureSets, // words 82-84, 119
pub en_cmd_ft_sets: EnabledCommandFeatureSets, // words 85-87, 120
pub current_udma_mode: SelectedUdmaMode, // word 88, bits 14:8
pub supported_udma_modes: SupportedUdmaModes, // word 88, bits 6:0
pub normal_sec_erasure_time: u16, // word 89
pub enhanced_sec_erasure_time: u16, // word 90
pub current_apm_level: u8, // word 91
pub master_pwd_identifier: u16, // word 92
pub hw_test_res: u16, // word 93
pub recommended_aam_level: u8, // word 94, bits 15:8
pub current_aam_level: u8, // word 94, bits 7:0
pub stream_min_req_size: u16, // word 95
pub dma_stream_transfer_time: u16, // word 96
pub stream_access_latency: u16, // word 97
pub stream_performance_granularity: u32, // words 98-99
pub total_sectors_lba48: u64, // words 100-103
pub pio_stream_transfer_time: u16, // word 104
pub logical_sectors_per_phys_sector: u16, // word 106
pub wwn: [u16; 4], // words 108-111
pub logical_sector_size: u32, // words 117-118
pub security_status: SecurityStatus, // word 128
pub cfa_power_mode: CFAPowerMode, // word 160
pub nominal_form_factor: u8, // word 168
pub current_media_sn: String, // words 176-205
pub sct: SCTCommandTransport, // word 206
pub logical_sector_alignment: u16, // word 209
pub wrv_count_mode3: u32, // words 210-211
pub wrv_count_mode2: u32, // words 212-213
pub nvcache_caps: NVCacheCapabilities, // word 214
pub nvcache_size_logical: u32, // words 215-216
pub nominal_rotation_rate: u16, // word 217
pub nvcache_retrieval_time: u8, // word 219
pub wrv_mode: u8, // word 220
pub transport_major: u16, //word 222
pub transport_minor: u16, // word 223
pub min_blocks_for_microcode: u16, // word 234
pub max_blocks_for_microcode: u16, // word 235
}


