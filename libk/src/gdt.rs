// SPDX-License-Identifier: MPL-2.0
use log::*;
use spin::Lazy;
use x86_64::instructions::{
    segmentation::{Segment, CS},
    tables::load_tss,
};
use x86_64::structures::{
    gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
    tss::TaskStateSegment,
};
use x86_64::VirtAddr;

/// Double-fault stack index
pub const DF_IST_IDX: u16 = 0;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DF_IST_IDX as usize] = {
        const STACK_SIZE: usize = 4096;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
        stack_start + STACK_SIZE
    };
    tss
});
static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (
        gdt,
        Selectors {
            code_selector,
            tss_selector,
        },
    )
});

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Sets up the GDT, separate kernel stack, and TSS.
#[cold]
pub fn init() {
    info!("Loading GDT");
    debug!("Loading GDT at addr {:p}: {:?}", &GDT.0, GDT.0);
    GDT.0.load();
    unsafe {
        info!("Setting CS");
        debug!(
            "CS at addr {:p}: {:?}",
            &GDT.1.code_selector, GDT.1.code_selector
        );
        CS::set_reg(GDT.1.code_selector);
        info!("Loading TSS");
        debug!(
            "TSS at addr {:p}, TSS selector at {:p}: TSS = {:?}, TSS selector = {:?}",
            &TSS, &GDT.1.tss_selector, TSS, GDT.1.tss_selector
        );
        load_tss(GDT.1.tss_selector);
    }
    debug!(
        "TSS at addr {:p} with TSS selecter at addr {:p} loaded",
        &TSS, &GDT.1.tss_selector
    );
}
