// SPDX-License-Identifier: MPL-2.0
// Log pages (see chapter 9 of ATA ACS-4)
// We omit pages 00h (List of supported pages) and 01h (Copy of IDENTIFY DEVICE data)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LogPageHeader {
pub page: u8,
pub revision: u16
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Capacity {
// Words 0 .. 8 - Capacity page information header.
pub header: LogPageHeader,
// words 8..15 - Device Capacity
pub capacity: u64, // bits 47:0
// Words 16..23 - Physical/Logical Sector Size
pub lps_supported: bool, // bit 62
pub lss_supported: bool, // bit 61
pub alignment_err_rep: u8, // bits 21:20
pub lps_sec_relationship: u8, // bits 19:16
pub log_sec_offset: u16, // bits 15:0
// Words 24..31 - Logical Sector Size
pub logical_sector_size: u32, // bits 31:0
// Words 32..39 - Nominal Buffer Size
pub nominal_buffer_size: u64, // bits62:0 (bit 63 is validity bit)
// Words 40..511 are reserved
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SupportedCapabilities {
// Words 0 .. 8 - Supported Capabilities page information header.
pub header: LogPageHeader,
// Words 8 .. 15 - Supported Capabilities
pub supports_abo: bool, // bit 54
pub persistent_sense_data_reporting: bool, // bit 53
pub sff8447_reporting: bool, // bit 52
pub definitive_ending_pattern: bool, // bit 51
pub supports_dsmxl: bool, // bit 50
pub supports_set_sector_configuration: bool, // bit 49
pub supports_zero_ext: bool, // bit 48
pub supports_succ_ncq_sense_data: bool, // bit 47
pub supports_dlc: bool, // bit 46
pub supports_rq_sense_df: bool, // bit 45
pub supports_dsn: bool, // bit 44
pub supports_low_power_standby: bool, // bit 43
pub supports_set_epc_pwr_src: bool, // bit 42
pub supports_amax_addr: bool, // bit 41
pub supports_drat: bool, // bit 39
pub supports_lps_misalignment_reporting: bool, // bit 38
pub supports_read_buffer_dma: bool, // bit 36
pub supports_write_buffer_dma: bool, // bit 35
pub supports_dm_dma: bool, // bit 33
pub optional_28bit: bool, // bit 32
pub supports_rzat: bool, // bit 31
pub supports_nop: bool, // bit 29
pub supports_read_buffer: bool, // bit 28
pub supports_write_buffer: bool, // bit 27
pub supports_read_lookahead: bool, // bit 25
pub supports_volatile_write_cache: bool, // bit 24
pub supports_smart: bool, // bit 23
pub supports_flush_cache_ext: bool, // bit 22
pub supports_48bit: bool, // bit 20
pub supports_spin_up: bool, // bit 18
pub supports_puis: bool, // bit 17
pub supports_apm: bool, // bit 16
pub supports_dm: bool, // bit 14
pub supports_unload: bool, // bit 13
pub supports_write_fua_ext: bool, // bit 12
pub supports_gpl: bool, // bit 11
pub supports_streaming: bool, // bit 10
pub supports_smart_self_test: bool, // bit 8
pub supports_smart_error_logging: bool, // bit 7
pub supports_epc: bool, // bit 6
pub supports_sense_data: bool, // bit 5
pub supports_free_fall: bool, // bit 4
pub supports_dm_mode3: bool, // bit 3
pub supports_gpl_dma: bool, // bit 2
pub supports_write_uncorrectable: bool, // bit 1
pub supports_wrv: bool, // bit 0
// Words 16..23 - DOWNLOAD MICROCODE Capabilities
pub dm_clears_nonactivated_deferred_data: bool, // bit 35
pub supports_dm_offsets: bool, // bit 34
pub supports_dm_imm: bool, // bit 33
pub supports_dm_imm_offsets: bool, // bit 32
pub dm_max_tx_size: u16, // bits 31:16
pub dm_min_tx_size: u16, // bits 16:0
// Words 24..31 - Nominal Media Rotation Rate
pub nominal_media_rotation_rate: u16, // bits 15:0
// Words 32..39 - Form Factor
pub form_factor: u8, // bits 3:0
// Words 40..47 - Write-Read-Verify Sector Count Mode 3
pub wrv_mode3_count: u32, // bits 31:0
// Words 48..55 - Write-Read-Verify Sector Count Mode 2
pub wrv_mode2_count: u32, // bits 31:0
// Words 56..71 - World wide name
pub wwn: String<U8>, // bits 63:0
// Words 72..79 - DATA SET MANAGEMENT
pub max_dsm_pages: u16, // bits 31:16
pub logical_blocks_markup: u8, // bits 15:8
pub supports_trim: bool, // bit 0
// Words 80..95 - Utilization Per Unit Time
pub utilization_type: u8, // bits 119:112
pub utilization_units: u8, // bits 111:104
pub utilization_interval: u8, // bits 103:96
pub utilization_b: u32, // bits 63:32
pub utilization_a: u32, // bits 32:0
// Words 96..103 - Utilization Usage Rate Support
pub supports_setting_rate_basis: bool, // bit 23
pub supports_since_po_rate_basis: bool, // bit 8
pub supports_po_rate_basis: bool, // bit 4
pub supports_dt_rate_basis: bool, // bit 0
// Words 104..111 - Zoned Capabilities
pub zoned: u8, // bits 1:0
// Words 112..119 - Supported ZAC Capabilities
pub supports_reset_write_pointers_ext: bool, // bit 4
pub supports_finish_zone_ext: bool, // bit 3
pub supports_close_zone_ext: bool, // bit 2
pub supports_open_zone_ext: bool, // bit 1
pub supports_report_zones_ext: bool, // bit 0
// Words 120..127 - Advanced Background Operations Capabilities
pub supports_abo_foreground: bool, // bit 62
pub supports_abo_ir: bool, // bit 61
pub abo_min_frac: u32, // bits 47:16
pub abo_min_tl: u16, // bits 15:0
// Words 128..135 - Advanced Background Operations Recommendations
pub dev_maint_pt: u16, // bits 31:16
pub rec_abo_start_interval: u16, // bits 15:0
// Words 136..143 - Queue Depth
pub queue_depth: u8, // bits 4:0
// Words 144..151 - Supported SCT Capabilities
pub supports_sctws103: bool, // bit 26
pub supports_sctws102: bool, // bit 25
pub supports_sctws101: bool, // bit 24
pub supports_sctws3: bool, // bit 18
pub supports_sctws2: bool, // bit 17
pub supports_sctws1: bool, // bit 16
pub supports_sct_data_tables: bool, // bit 5
pub supports_sct_feature_ctl: bool, // bit 4
pub supports_sct_err_rec: bool, // bit 3
pub supports_sctws: bool, // bit 2
pub supports_sct: bool, // bit 0
// Words 152..159 - Depopulation Capabilities
pub supports_get_phys_elmt_sts: bool, // bit 1
pub supports_rm_elmt_and_trunc: bool, // bit 0
// Words 160..167 - Depopulation Execution Time
pub depopulation_time: u64, // bits 62:0
// Words 168..503 are reserved
// Words 504..511 - Vendor Specific Supported Capabilities
pub vendor_caps: u64, // all bits
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CurrentSettings {
// Words 0..8 - Current Settings page information header.
pub header: LogPageHeader,
// Words 8..15 - Current Settings
pub fw_activation_pending: bool, // bit 19
pub succ_ncq_cmd_sense_data: bool, // bit 18
pub dlc: bool, // bit 17
pub dsn: bool, // bit 16
pub epc: bool, // bit 15
pub volatile_write_cache: bool, // bit 13
pub reverting_to_defaults: bool, // bit 11
pub sense_data: bool, // bit 10
pub nonvolatile_write_cache: bool, // bit 8
pub read_lookahead: bool, // bit 7
pub smart: bool, // bit 6
pub puis: bool, // bit 3
pub apm: bool, // bit 2
pub free_fall: bool, // bit 1
pub wrv: bool, // bit 0
// Words 16..23 - Feature Settings
pub pwr_src: u8, // bits 17:16
pub apm_lvl: u8, // bits 15:8
pub wrv_mode: u8, // bits 7:0
// Words 24..31 - DMA Host Interface Sector Times
pub dma_sector_time: u16, // bits 15:0
// Words 32..39 - PIO Host Interface Sector Times
pub pio_sector_times: u16, // bits 15:0
// Words 40..47 - Streaming minimum request size
pub stream_min_request_size: u16, // bits 15:0
// Words 48..55 - Streaming access latency
pub stream_access_latency: u16, // bits 15:0
// Words 56..63 - Streaming Performance Granularity
pub stream_granularity: u32, // bits 31:0
// Words 64..71 - Free-fall Control Sensitivity
pub free_fall_sensitivity: u8, // bits 7:0
// Words 72..79 - Device Maintenance Schedule
pub min_inactive_time_ms: u16, // bits 57:48
pub dev_maint_time: u16, // bits 47:32
pub performance_deg_time: u16, // bits 31:16
pub min_inactive_time: u16, // bits 15:0
// Words 80..87 - Advanced Background Operations Settings
pub abo_status: u8, // bits 7:0
}
