pub struct APICBase {
    pub bsp: bool,
    pub enable_x2apic: bool,
    pub apic_global_enable: bool,
    pub apic_addr: usize,
}

pub struct FeatureControl {
    pub lock: bool,
    pub enable_vmx_inside_smx: bool,
    pub enable_vmx_outside_smx: bool,
    pub senter_local_enables: [bool; 6],
    pub senter_global_enable: bool,
    pub sgx_launch_control_enable: bool,
    pub sgx_global_enable: bool,
    pub lmce_on: bool,
}

pub struct SpecCtrl {
    pub ibrs: bool,
    pub stibp: bool,
    pub ssbd: bool,
}

pub struct SmmMonitorCtl {
    pub valid: bool,
    pub unblock_smi_on_vmxoff: bool,
    pub mseg_base: u32,
}

pub struct UmwaitControl {
    pub c02: bool,
    pub tsc_quanta: u32,
}

pub struct MtrrCap {
    pub vcnt: u8,
    pub fixed_range_mtrrs: bool,
    pub wc: bool,
    pub smrr: bool,
    pub prmrr: bool,
}
