// SPDX-License-Identifier: MPL-2.0
use log::*;
use spin::Lazy;
use x86_64::instructions::{segmentation::set_cs, tables::load_tss};
use x86_64::structures::{
    gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
    tss::TaskStateSegment,
};
use x86_64::VirtAddr;

/// Double-fault stack index
pub const DF_IST_IDX: u16 = 0;
/// Breakpoint stack index.
pub const BP_IST_IDX: u16 = 1;
/// Page fault stack index.
pub const PF_IST_IDX: u16 = 2;
/// Overflow stack index.
pub const OF_IST_IDX: u16 = 3;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DF_IST_IDX as usize] = {
        const STACK_SIZE: usize = 4096;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
        stack_start + STACK_SIZE
    };
    tss.interrupt_stack_table[BP_IST_IDX as usize] = {
        const STACK_SIZE: usize = 65536;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
        stack_start + STACK_SIZE
    };
    tss.interrupt_stack_table[PF_IST_IDX as usize] = {
        const STACK_SIZE: usize = 262144;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
        stack_start + STACK_SIZE
    };
    tss.interrupt_stack_table[PF_IST_IDX as usize] = {
        const STACK_SIZE: usize = 32768;
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
    GDT.0.load();
    unsafe {
        info!("Setting CS");
        set_cs(GDT.1.code_selector);
        info!("Loading TSS");
        load_tss(GDT.1.tss_selector);
    }
}
