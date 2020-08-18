// SPDX-License-Identifier: MPL-2.0
use lazy_static::lazy_static;
use x86_64::instructions::segmentation::set_cs;
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use crate::printkln;

pub const DF_IST_IDX: u16 = 0;
pub const BP_IST_IDX: u16 = 1;
pub const PF_IST_IDX: u16 = 2;
pub const OF_IST_IDX: u16 = 3;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DF_IST_IDX as usize] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            stack_start + STACK_SIZE
        };
        tss
    };
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
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
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Sets up the GDT, separate kernel stack, and TSS.
pub async fn init() {
printkln!("init: gdt: loading");
    GDT.0.load();
    unsafe {
    printkln!("Init: GDT: Setting CS");
        set_cs(GDT.1.code_selector);
        printkln!("Init: GDT: Loading TSS");
        load_tss(GDT.1.tss_selector);
    }
}
