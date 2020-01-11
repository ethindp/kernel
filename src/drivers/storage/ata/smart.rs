#[repr(u8)]
pub enum ExecuteOfflineImmediateSubcommand {
    /// Execute SMART off-line routine immediately in off-line mode
    OfflineOffline = 0x00,
    /// Execute SMART Short self-test routine immediately in off-line mode
    ShortSelfTestOffline = 0x01,
    /// Execute SMART Extended self-test routine immediately in off-line mode
    ExtendedSelfTestOffline = 0x02,
    /// Execute SMART Conveyance self-test routine immediately in off-line mode
    ConveyanceSelfTestOffline = 0x03,
    /// Execute SMART Selective self-test routine immediately in off-line mode
    SelectiveSelfTestOffline = 0x04,
    /// Abort off-line mode self-test routine
    AbortOffline = 0x7F,
    /// Execute SMART Short self-test routine immediately in captive mode
    ShortSelfTestCaptive = 0x81,
    /// Execute SMART Extended self-test routine immediately in captive mode
    ExtendedSelfTestCaptive = 0x82,
    /// Execute SMART Conveyance self-test routine immediately in captive mode
    ConveyanceSelfTestCaptive = 0x83,
    /// Execute SMART Selective self-test routine immediately in captive mode
    SelectiveSelfTestCaptive = 0x84,
}
