// SPDX-License-Identifier: MPL-2.0
use crate::acpi::get_hpet_info;
use crate::gdt;
use alloc::boxed::Box;
use bit_field::BitField;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use heapless::FnvIndexMap;
use log::*;
use minivec::MiniVec;
use raw_cpuid::*;
use spin::{Lazy, RwLock};
use voladdress::*;
use x86_64::{
    instructions::hlt,
    registers::model_specific::Msr,
    structures::idt::{
        InterruptDescriptorTable, InterruptStackFrame, InterruptStackFrameValue, PageFaultErrorCode,
    },
};

/// Types to contain IRQ functions and interrupt handlers
type IrqList = FnvIndexMap<u8, MiniVec<InterruptHandler>, 256>;
/// This is the type for interrupt handlers.
pub type InterruptHandler = Box<dyn Fn(InterruptStackFrameValue) + Send + Sync>;

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    // Handle BPs
    let _ = idt.breakpoint.set_handler_fn(handle_breakpoint);
    // Handle DFs (on our set-up separate 4K kernel stack)
    unsafe {
        let _ = idt
            .double_fault
            .set_handler_fn(handle_double_fault)
            .set_stack_index(gdt::DF_IST_IDX);
    }
    let _ = idt.page_fault.set_handler_fn(handle_page_fault);
    let _ = idt.overflow.set_handler_fn(handle_overflow);
    let _ = idt
        .bound_range_exceeded
        .set_handler_fn(handle_bound_range_exceeded);
    let _ = idt.invalid_opcode.set_handler_fn(handle_invalid_opcode);
    let _ = idt
        .device_not_available
        .set_handler_fn(handle_device_not_available);
    let _ = idt
        .general_protection_fault
        .set_handler_fn(handle_general_protection_fault);
    let _ = idt.alignment_check.set_handler_fn(handle_alignment_check);
    let _ = idt.debug.set_handler_fn(handle_debug);
    let _ = idt.divide_error.set_handler_fn(handle_divide_error);
    let _ = idt
        .non_maskable_interrupt
        .set_handler_fn(handle_non_maskable_interrupt);
    let _ = idt.invalid_tss.set_handler_fn(handle_invalid_tss);
    let _ = idt
        .segment_not_present
        .set_handler_fn(handle_segment_not_present);
    let _ = idt
        .stack_segment_fault
        .set_handler_fn(handle_stack_segment_fault);
    let _ = idt
        .x87_floating_point
        .set_handler_fn(handle_x87_floating_point);
    let _ = idt.machine_check.set_handler_fn(handle_machine_check);
    let _ = idt
        .simd_floating_point
        .set_handler_fn(handle_simd_floating_point);
    let _ = idt
        .virtualization
        .set_handler_fn(handle_virtualization_exception);
    let _ = idt
        .security_exception
        .set_handler_fn(handle_security_exception);
    let _ = idt[32].set_handler_fn(handle_timer);
    let _ = idt[33].set_handler_fn(handle_keyboard);
    let _ = idt[34].set_handler_fn(handle_cascade);
    let _ = idt[35].set_handler_fn(handle_uart1);
    let _ = idt[36].set_handler_fn(handle_serial1);
    let _ = idt[37].set_handler_fn(handle_parallel);
    let _ = idt[38].set_handler_fn(handle_floppy);
    let _ = idt[39].set_handler_fn(handle_lpt1);
    let _ = idt[40].set_handler_fn(handle_rtc);
    let _ = idt[41].set_handler_fn(handle_acpi);
    let _ = idt[42].set_handler_fn(handle_open1);
    let _ = idt[43].set_handler_fn(handle_open2);
    let _ = idt[44].set_handler_fn(handle_mouse);
    let _ = idt[45].set_handler_fn(handle_coprocessor);
    let _ = idt[46].set_handler_fn(handle_primary_ata);
    let _ = idt[47].set_handler_fn(handle_secondary_ata);
    let _ = idt[48].set_handler_fn(handle_irq48);
    let _ = idt[49].set_handler_fn(handle_irq49);
    let _ = idt[50].set_handler_fn(handle_irq50);
    let _ = idt[51].set_handler_fn(handle_irq51);
    let _ = idt[52].set_handler_fn(handle_irq52);
    let _ = idt[53].set_handler_fn(handle_irq53);
    let _ = idt[54].set_handler_fn(handle_irq54);
    let _ = idt[55].set_handler_fn(handle_irq55);
    let _ = idt[56].set_handler_fn(handle_irq56);
    let _ = idt[57].set_handler_fn(handle_irq57);
    let _ = idt[58].set_handler_fn(handle_irq58);
    let _ = idt[59].set_handler_fn(handle_irq59);
    let _ = idt[60].set_handler_fn(handle_irq60);
    let _ = idt[61].set_handler_fn(handle_irq61);
    let _ = idt[62].set_handler_fn(handle_irq62);
    let _ = idt[63].set_handler_fn(handle_irq63);
    let _ = idt[64].set_handler_fn(handle_irq64);
    let _ = idt[65].set_handler_fn(handle_irq65);
    let _ = idt[66].set_handler_fn(handle_irq66);
    let _ = idt[67].set_handler_fn(handle_irq67);
    let _ = idt[68].set_handler_fn(handle_irq68);
    let _ = idt[69].set_handler_fn(handle_irq69);
    let _ = idt[70].set_handler_fn(handle_irq70);
    let _ = idt[71].set_handler_fn(handle_irq71);
    let _ = idt[72].set_handler_fn(handle_irq72);
    let _ = idt[73].set_handler_fn(handle_irq73);
    let _ = idt[74].set_handler_fn(handle_irq74);
    let _ = idt[75].set_handler_fn(handle_irq75);
    let _ = idt[76].set_handler_fn(handle_irq76);
    let _ = idt[77].set_handler_fn(handle_irq77);
    let _ = idt[78].set_handler_fn(handle_irq78);
    let _ = idt[79].set_handler_fn(handle_irq79);
    let _ = idt[80].set_handler_fn(handle_irq80);
    let _ = idt[81].set_handler_fn(handle_irq81);
    let _ = idt[82].set_handler_fn(handle_irq82);
    let _ = idt[83].set_handler_fn(handle_irq83);
    let _ = idt[84].set_handler_fn(handle_irq84);
    let _ = idt[85].set_handler_fn(handle_irq85);
    let _ = idt[86].set_handler_fn(handle_irq86);
    let _ = idt[87].set_handler_fn(handle_irq87);
    let _ = idt[88].set_handler_fn(handle_irq88);
    let _ = idt[89].set_handler_fn(handle_irq89);
    let _ = idt[90].set_handler_fn(handle_irq90);
    let _ = idt[91].set_handler_fn(handle_irq91);
    let _ = idt[92].set_handler_fn(handle_irq92);
    let _ = idt[93].set_handler_fn(handle_irq93);
    let _ = idt[94].set_handler_fn(handle_irq94);
    let _ = idt[95].set_handler_fn(handle_irq95);
    let _ = idt[96].set_handler_fn(handle_irq96);
    let _ = idt[97].set_handler_fn(handle_irq97);
    let _ = idt[98].set_handler_fn(handle_irq98);
    let _ = idt[99].set_handler_fn(handle_irq99);
    let _ = idt[100].set_handler_fn(handle_irq100);
    let _ = idt[101].set_handler_fn(handle_irq101);
    let _ = idt[102].set_handler_fn(handle_irq102);
    let _ = idt[103].set_handler_fn(handle_irq103);
    let _ = idt[104].set_handler_fn(handle_irq104);
    let _ = idt[105].set_handler_fn(handle_irq105);
    let _ = idt[106].set_handler_fn(handle_irq106);
    let _ = idt[107].set_handler_fn(handle_irq107);
    let _ = idt[108].set_handler_fn(handle_irq108);
    let _ = idt[109].set_handler_fn(handle_irq109);
    let _ = idt[110].set_handler_fn(handle_irq110);
    let _ = idt[111].set_handler_fn(handle_irq111);
    let _ = idt[112].set_handler_fn(handle_irq112);
    let _ = idt[113].set_handler_fn(handle_irq113);
    let _ = idt[114].set_handler_fn(handle_irq114);
    let _ = idt[115].set_handler_fn(handle_irq115);
    let _ = idt[116].set_handler_fn(handle_irq116);
    let _ = idt[117].set_handler_fn(handle_irq117);
    let _ = idt[118].set_handler_fn(handle_irq118);
    let _ = idt[119].set_handler_fn(handle_irq119);
    let _ = idt[120].set_handler_fn(handle_irq120);
    let _ = idt[121].set_handler_fn(handle_irq121);
    let _ = idt[122].set_handler_fn(handle_irq122);
    let _ = idt[123].set_handler_fn(handle_irq123);
    let _ = idt[124].set_handler_fn(handle_irq124);
    let _ = idt[125].set_handler_fn(handle_irq125);
    let _ = idt[126].set_handler_fn(handle_irq126);
    let _ = idt[127].set_handler_fn(handle_irq127);
    let _ = idt[128].set_handler_fn(handle_irq128);
    let _ = idt[129].set_handler_fn(handle_irq129);
    let _ = idt[130].set_handler_fn(handle_irq130);
    let _ = idt[131].set_handler_fn(handle_irq131);
    let _ = idt[132].set_handler_fn(handle_irq132);
    let _ = idt[133].set_handler_fn(handle_irq133);
    let _ = idt[134].set_handler_fn(handle_irq134);
    let _ = idt[135].set_handler_fn(handle_irq135);
    let _ = idt[136].set_handler_fn(handle_irq136);
    let _ = idt[137].set_handler_fn(handle_irq137);
    let _ = idt[138].set_handler_fn(handle_irq138);
    let _ = idt[139].set_handler_fn(handle_irq139);
    let _ = idt[140].set_handler_fn(handle_irq140);
    let _ = idt[141].set_handler_fn(handle_irq141);
    let _ = idt[142].set_handler_fn(handle_irq142);
    let _ = idt[143].set_handler_fn(handle_irq143);
    let _ = idt[144].set_handler_fn(handle_irq144);
    let _ = idt[145].set_handler_fn(handle_irq145);
    let _ = idt[146].set_handler_fn(handle_irq146);
    let _ = idt[147].set_handler_fn(handle_irq147);
    let _ = idt[148].set_handler_fn(handle_irq148);
    let _ = idt[149].set_handler_fn(handle_irq149);
    let _ = idt[150].set_handler_fn(handle_irq150);
    let _ = idt[151].set_handler_fn(handle_irq151);
    let _ = idt[152].set_handler_fn(handle_irq152);
    let _ = idt[153].set_handler_fn(handle_irq153);
    let _ = idt[154].set_handler_fn(handle_irq154);
    let _ = idt[155].set_handler_fn(handle_irq155);
    let _ = idt[156].set_handler_fn(handle_irq156);
    let _ = idt[157].set_handler_fn(handle_irq157);
    let _ = idt[158].set_handler_fn(handle_irq158);
    let _ = idt[159].set_handler_fn(handle_irq159);
    let _ = idt[160].set_handler_fn(handle_irq160);
    let _ = idt[161].set_handler_fn(handle_irq161);
    let _ = idt[162].set_handler_fn(handle_irq162);
    let _ = idt[163].set_handler_fn(handle_irq163);
    let _ = idt[164].set_handler_fn(handle_irq164);
    let _ = idt[165].set_handler_fn(handle_irq165);
    let _ = idt[166].set_handler_fn(handle_irq166);
    let _ = idt[167].set_handler_fn(handle_irq167);
    let _ = idt[168].set_handler_fn(handle_irq168);
    let _ = idt[169].set_handler_fn(handle_irq169);
    let _ = idt[170].set_handler_fn(handle_irq170);
    let _ = idt[171].set_handler_fn(handle_irq171);
    let _ = idt[172].set_handler_fn(handle_irq172);
    let _ = idt[173].set_handler_fn(handle_irq173);
    let _ = idt[174].set_handler_fn(handle_irq174);
    let _ = idt[175].set_handler_fn(handle_irq175);
    let _ = idt[176].set_handler_fn(handle_irq176);
    let _ = idt[177].set_handler_fn(handle_irq177);
    let _ = idt[178].set_handler_fn(handle_irq178);
    let _ = idt[179].set_handler_fn(handle_irq179);
    let _ = idt[180].set_handler_fn(handle_irq180);
    let _ = idt[181].set_handler_fn(handle_irq181);
    let _ = idt[182].set_handler_fn(handle_irq182);
    let _ = idt[183].set_handler_fn(handle_irq183);
    let _ = idt[184].set_handler_fn(handle_irq184);
    let _ = idt[185].set_handler_fn(handle_irq185);
    let _ = idt[186].set_handler_fn(handle_irq186);
    let _ = idt[187].set_handler_fn(handle_irq187);
    let _ = idt[188].set_handler_fn(handle_irq188);
    let _ = idt[189].set_handler_fn(handle_irq189);
    let _ = idt[190].set_handler_fn(handle_irq190);
    let _ = idt[191].set_handler_fn(handle_irq191);
    let _ = idt[192].set_handler_fn(handle_irq192);
    let _ = idt[193].set_handler_fn(handle_irq193);
    let _ = idt[194].set_handler_fn(handle_irq194);
    let _ = idt[195].set_handler_fn(handle_irq195);
    let _ = idt[196].set_handler_fn(handle_irq196);
    let _ = idt[197].set_handler_fn(handle_irq197);
    let _ = idt[198].set_handler_fn(handle_irq198);
    let _ = idt[199].set_handler_fn(handle_irq199);
    let _ = idt[200].set_handler_fn(handle_irq200);
    let _ = idt[201].set_handler_fn(handle_irq201);
    let _ = idt[202].set_handler_fn(handle_irq202);
    let _ = idt[203].set_handler_fn(handle_irq203);
    let _ = idt[204].set_handler_fn(handle_irq204);
    let _ = idt[205].set_handler_fn(handle_irq205);
    let _ = idt[206].set_handler_fn(handle_irq206);
    let _ = idt[207].set_handler_fn(handle_irq207);
    let _ = idt[208].set_handler_fn(handle_irq208);
    let _ = idt[209].set_handler_fn(handle_irq209);
    let _ = idt[210].set_handler_fn(handle_irq210);
    let _ = idt[211].set_handler_fn(handle_irq211);
    let _ = idt[212].set_handler_fn(handle_irq212);
    let _ = idt[213].set_handler_fn(handle_irq213);
    let _ = idt[214].set_handler_fn(handle_irq214);
    let _ = idt[215].set_handler_fn(handle_irq215);
    let _ = idt[216].set_handler_fn(handle_irq216);
    let _ = idt[217].set_handler_fn(handle_irq217);
    let _ = idt[218].set_handler_fn(handle_irq218);
    let _ = idt[219].set_handler_fn(handle_irq219);
    let _ = idt[220].set_handler_fn(handle_irq220);
    let _ = idt[221].set_handler_fn(handle_irq221);
    let _ = idt[222].set_handler_fn(handle_irq222);
    let _ = idt[223].set_handler_fn(handle_irq223);
    let _ = idt[224].set_handler_fn(handle_irq224);
    let _ = idt[225].set_handler_fn(handle_irq225);
    let _ = idt[226].set_handler_fn(handle_irq226);
    let _ = idt[227].set_handler_fn(handle_irq227);
    let _ = idt[228].set_handler_fn(handle_irq228);
    let _ = idt[229].set_handler_fn(handle_irq229);
    let _ = idt[230].set_handler_fn(handle_irq230);
    let _ = idt[231].set_handler_fn(handle_irq231);
    let _ = idt[232].set_handler_fn(handle_irq232);
    let _ = idt[233].set_handler_fn(handle_irq233);
    let _ = idt[234].set_handler_fn(handle_irq234);
    let _ = idt[235].set_handler_fn(handle_irq235);
    let _ = idt[236].set_handler_fn(handle_irq236);
    let _ = idt[237].set_handler_fn(handle_irq237);
    let _ = idt[238].set_handler_fn(handle_irq238);
    let _ = idt[239].set_handler_fn(handle_irq239);
    let _ = idt[240].set_handler_fn(handle_irq240);
    let _ = idt[241].set_handler_fn(handle_irq241);
    let _ = idt[242].set_handler_fn(handle_irq242);
    let _ = idt[243].set_handler_fn(handle_irq243);
    let _ = idt[244].set_handler_fn(handle_irq244);
    let _ = idt[245].set_handler_fn(handle_irq245);
    let _ = idt[246].set_handler_fn(handle_irq246);
    let _ = idt[247].set_handler_fn(handle_irq247);
    let _ = idt[248].set_handler_fn(handle_irq248);
    let _ = idt[249].set_handler_fn(handle_irq249);
    let _ = idt[250].set_handler_fn(handle_irq250);
    let _ = idt[251].set_handler_fn(handle_irq251);
    let _ = idt[252].set_handler_fn(handle_irq252);
    let _ = idt[253].set_handler_fn(handle_irq253);
    let _ = idt[254].set_handler_fn(handle_irq254);
    let _ = idt[255].set_handler_fn(handle_irq255);
    idt
});
static TICK_COUNT: AtomicU64 = AtomicU64::new(0);
static IRQ_FUNCS: Lazy<RwLock<IrqList>> = Lazy::new(|| {
    let mut table = IrqList::new();
    (0..u8::MAX).for_each(|i| {
        let v = MiniVec::<InterruptHandler>::new();
        if table.insert(i, v).is_err() {
            panic!("Cannot add ISR function table for interrupt {}!", i);
        }
    });
    RwLock::new(table)
});
static X2APIC: AtomicBool = AtomicBool::new(false);
static APIC: AtomicBool = AtomicBool::new(false);

/// Initializes either the APIC or X2APIC
pub fn init_ic() {
    info!("Disabling interrupts");
    x86_64::instructions::interrupts::disable();
    info!("Checking for APIC");
    if is_apic_available() {
        let id = CpuId::new();
        let feature_info = id.get_feature_info().unwrap();
        if !feature_info.has_x2apic() {
            panic!("X2APIC is required, but system only has XAPIC");
        } else {
            info!("Configuring X2APIC");
            let mut ia32_apic_base = Msr::new(0x01B);
            unsafe {
                let mut apic_base = ia32_apic_base.read();
                apic_base.set_bit(10, true);
                apic_base.set_bit(11, true);
                ia32_apic_base.write(apic_base);
            }
            info!("X2apic configured");
        }
    } else {
        panic!("APIC/X2APIC not available/supported");
    }
    info!("Enabling interrupts");
    x86_64::instructions::interrupts::enable();
}

/// Loads the IDT
pub fn init_idt() {
    use x86_64::instructions::tables::sidt;
    let oldidtr = sidt();
    info!("Loading IDT");
    IDT.load();
    debug!("IDT loaded. IDT at {:p}: {:?}", &IDT, IDT);
    let newidtr = sidt();
    let oldlimit = oldidtr.limit;
    let newlimit = newidtr.limit;
    debug!(
        "Changed IDT: old: {:X} with limit {:X}, new: {:X} with limit {:X}",
        oldidtr.base.as_u64(),
        oldlimit,
        newidtr.base.as_u64(),
        newlimit
    );
}

// Macro to generate interrupt functions
macro_rules! gen_interrupt_fn {
    ($i:ident, $p:expr) => {
        extern "x86-interrupt" fn $i(stack_frame: InterruptStackFrame) {
            signal_eoi();
            debug!("Interrupt received for int {}", $p);
            IRQ_FUNCS
                .read()
                .get(&$p)
                .unwrap()
                .iter()
                .enumerate()
                .for_each(|(i, func)| {
                    debug!("Calling func {:X}", i);
                    (func)(stack_frame.clone());
                });
        }
    };
}

extern "x86-interrupt" fn handle_breakpoint(stack_frame: InterruptStackFrame) {
    // All we do here is notify the user and continue on.
    info!(
        "Hardware breakpoint interrupt received:\n{:#?}",
        stack_frame
    );
    signal_eoi();
}

extern "x86-interrupt" fn handle_double_fault(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT({})\n{:#?}",
        error_code, stack_frame,
    );
}

extern "x86-interrupt" fn handle_timer(_s: InterruptStackFrame) {
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    signal_eoi();
}

extern "x86-interrupt" fn handle_rtc(_stack_frame: InterruptStackFrame) {
    signal_eoi();
    unsafe {
        use x86_64::instructions::port::Port;
        Port::<u8>::new(0x70).write(0x0C);
        Port::<u8>::new(0x71).read();
    }
}

extern "x86-interrupt" fn handle_page_fault(
    frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use crate::idle_forever;
    use x86_64::registers::control::Cr2;
    let addr = Cr2::read();
    error!(
        "Page fault: {} while {} memory address {:X}h",
        if error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
            "protection violation"
        } else if !error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
            "page not present"
        } else if error_code.contains(PageFaultErrorCode::USER_MODE) {
            "UM priv violation"
        } else if !error_code.contains(PageFaultErrorCode::USER_MODE) {
            "KM priv violation"
        } else if error_code.contains(PageFaultErrorCode::MALFORMED_TABLE) {
            "PTT read failure"
        } else if error_code.contains(PageFaultErrorCode::INSTRUCTION_FETCH) {
            "Instruction fetch"
        } else if error_code.contains(PageFaultErrorCode::PROTECTION_KEY) {
            "protection key access"
        } else if error_code.contains(PageFaultErrorCode::SHADOW_STACK) {
            "shadow stack access"
        } else if error_code.contains(PageFaultErrorCode::SGX) {
            "SGX access control violation"
        } else if error_code.contains(PageFaultErrorCode::RMP) {
            "RMP violation"
        } else {
            "unknown cause"
        },
        if error_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE) {
            "writing to"
        } else {
            "reading from"
        },
        addr.as_u64()
    );
    error!(
        "Details:\nRegisters: RIP = {:X}\tCS = {:X}\tflags = {:X}\tRSP = {:X}\tSS = {:X}",
        frame.instruction_pointer.as_u64(),
        frame.code_segment,
        frame.cpu_flags,
        frame.stack_pointer.as_u64(),
        frame.stack_segment
    );
    idle_forever();
}

extern "x86-interrupt" fn handle_overflow(_: InterruptStackFrame) {
    warn!("Can't execute calculation: overflow");
    signal_eoi();
}

extern "x86-interrupt" fn handle_bound_range_exceeded(stack: InterruptStackFrame) {
    panic!(
        "Cannot continue: bounds range exceeded.\nStack:\n{:?}",
        stack,
    );
}

extern "x86-interrupt" fn handle_invalid_opcode(stack: InterruptStackFrame) {
    panic!("Cannot continue: invalid opcode!\nStack:\n{:?}", stack,);
}

extern "x86-interrupt" fn handle_device_not_available(stack: InterruptStackFrame) {
    panic!("Can't continue: device unavailable!\nStack:\n{:?}", stack,);
}

extern "x86-interrupt" fn handle_general_protection_fault(frame: InterruptStackFrame, ec: u64) {
    use crate::idle_forever;
    error!("Cannot continue (GP), error code {:X}", ec);
    error!(
        "Details:\nRegisters: RIP = {:X}\tCS = {:X}\tflags = {:X}\tRSP = {:X}\tSS = {:X}",
        frame.instruction_pointer.as_u64(),
        frame.code_segment,
        frame.cpu_flags,
        frame.stack_pointer.as_u64(),
        frame.stack_segment
    );
    idle_forever();
}

extern "x86-interrupt" fn handle_divide_error(frame: InterruptStackFrame) {
    panic!("Division error: {:?}", frame);
}

extern "x86-interrupt" fn handle_debug(_: InterruptStackFrame) {
    info!("Debug exception triggered");
    signal_eoi();
}

extern "x86-interrupt" fn handle_non_maskable_interrupt(_: InterruptStackFrame) {
    signal_eoi();
}

extern "x86-interrupt" fn handle_invalid_tss(frame: InterruptStackFrame, error_code: u64) {
    panic!("Invalid TSS: SS {:X}: {:?}", error_code, frame);
}

extern "x86-interrupt" fn handle_segment_not_present(frame: InterruptStackFrame, error_code: u64) {
    panic!("Segment not present: segment {:X}: {:?}", error_code, frame);
}

extern "x86-interrupt" fn handle_stack_segment_fault(frame: InterruptStackFrame, error_code: u64) {
    panic!("Stack-segment fault: segment {:X}: {:?}", error_code, frame);
}

extern "x86-interrupt" fn handle_x87_floating_point(frame: InterruptStackFrame) {
    panic!(
        "Impossible error: x87-floating-point exception! {:?}",
        frame
    );
}

extern "x86-interrupt" fn handle_alignment_check(frame: InterruptStackFrame, _: u64) {
    panic!("Alignment check exception: {:?}", frame);
}

extern "x86-interrupt" fn handle_machine_check(frame: InterruptStackFrame) -> ! {
    panic!("Machine check exception: {:?}", frame);
}

extern "x86-interrupt" fn handle_simd_floating_point(_: InterruptStackFrame) {
    panic!("SIMD FP error");
}

extern "x86-interrupt" fn handle_virtualization_exception(f: InterruptStackFrame) {
    panic!("Virtualization exception: {:?}", f);
}

extern "x86-interrupt" fn handle_security_exception(f: InterruptStackFrame, error_code: u64) {
    use crate::idle_forever;
    error!("Security exception");
    if error_code == 1 {
        error!("Detected redirection of INIT signal");
    }
    error!("Stack frame: {:?}", f);
    idle_forever();
}

gen_interrupt_fn!(handle_keyboard, 33);
gen_interrupt_fn!(handle_cascade, 34);
gen_interrupt_fn!(handle_uart1, 35);
gen_interrupt_fn!(handle_serial1, 36);
gen_interrupt_fn!(handle_parallel, 37);
gen_interrupt_fn!(handle_floppy, 38);
gen_interrupt_fn!(handle_lpt1, 39);
gen_interrupt_fn!(handle_acpi, 41);
gen_interrupt_fn!(handle_open1, 42);
gen_interrupt_fn!(handle_open2, 43);
gen_interrupt_fn!(handle_mouse, 44);
gen_interrupt_fn!(handle_coprocessor, 45);
gen_interrupt_fn!(handle_primary_ata, 46);
gen_interrupt_fn!(handle_secondary_ata, 47);
gen_interrupt_fn!(handle_irq48, 48);
gen_interrupt_fn!(handle_irq49, 49);
gen_interrupt_fn!(handle_irq50, 50);
gen_interrupt_fn!(handle_irq51, 51);
gen_interrupt_fn!(handle_irq52, 52);
gen_interrupt_fn!(handle_irq53, 53);
gen_interrupt_fn!(handle_irq54, 54);
gen_interrupt_fn!(handle_irq55, 55);
gen_interrupt_fn!(handle_irq56, 56);
gen_interrupt_fn!(handle_irq57, 57);
gen_interrupt_fn!(handle_irq58, 58);
gen_interrupt_fn!(handle_irq59, 59);
gen_interrupt_fn!(handle_irq60, 60);
gen_interrupt_fn!(handle_irq61, 61);
gen_interrupt_fn!(handle_irq62, 62);
gen_interrupt_fn!(handle_irq63, 63);
gen_interrupt_fn!(handle_irq64, 64);
gen_interrupt_fn!(handle_irq65, 65);
gen_interrupt_fn!(handle_irq66, 66);
gen_interrupt_fn!(handle_irq67, 67);
gen_interrupt_fn!(handle_irq68, 68);
gen_interrupt_fn!(handle_irq69, 69);
gen_interrupt_fn!(handle_irq70, 70);
gen_interrupt_fn!(handle_irq71, 71);
gen_interrupt_fn!(handle_irq72, 72);
gen_interrupt_fn!(handle_irq73, 73);
gen_interrupt_fn!(handle_irq74, 74);
gen_interrupt_fn!(handle_irq75, 75);
gen_interrupt_fn!(handle_irq76, 76);
gen_interrupt_fn!(handle_irq77, 77);
gen_interrupt_fn!(handle_irq78, 78);
gen_interrupt_fn!(handle_irq79, 79);
gen_interrupt_fn!(handle_irq80, 80);
gen_interrupt_fn!(handle_irq81, 81);
gen_interrupt_fn!(handle_irq82, 82);
gen_interrupt_fn!(handle_irq83, 83);
gen_interrupt_fn!(handle_irq84, 84);
gen_interrupt_fn!(handle_irq85, 85);
gen_interrupt_fn!(handle_irq86, 86);
gen_interrupt_fn!(handle_irq87, 87);
gen_interrupt_fn!(handle_irq88, 88);
gen_interrupt_fn!(handle_irq89, 89);
gen_interrupt_fn!(handle_irq90, 90);
gen_interrupt_fn!(handle_irq91, 91);
gen_interrupt_fn!(handle_irq92, 92);
gen_interrupt_fn!(handle_irq93, 93);
gen_interrupt_fn!(handle_irq94, 94);
gen_interrupt_fn!(handle_irq95, 95);
gen_interrupt_fn!(handle_irq96, 96);
gen_interrupt_fn!(handle_irq97, 97);
gen_interrupt_fn!(handle_irq98, 98);
gen_interrupt_fn!(handle_irq99, 99);
gen_interrupt_fn!(handle_irq100, 100);
gen_interrupt_fn!(handle_irq101, 101);
gen_interrupt_fn!(handle_irq102, 102);
gen_interrupt_fn!(handle_irq103, 103);
gen_interrupt_fn!(handle_irq104, 104);
gen_interrupt_fn!(handle_irq105, 105);
gen_interrupt_fn!(handle_irq106, 106);
gen_interrupt_fn!(handle_irq107, 107);
gen_interrupt_fn!(handle_irq108, 108);
gen_interrupt_fn!(handle_irq109, 109);
gen_interrupt_fn!(handle_irq110, 110);
gen_interrupt_fn!(handle_irq111, 111);
gen_interrupt_fn!(handle_irq112, 112);
gen_interrupt_fn!(handle_irq113, 113);
gen_interrupt_fn!(handle_irq114, 114);
gen_interrupt_fn!(handle_irq115, 115);
gen_interrupt_fn!(handle_irq116, 116);
gen_interrupt_fn!(handle_irq117, 117);
gen_interrupt_fn!(handle_irq118, 118);
gen_interrupt_fn!(handle_irq119, 119);
gen_interrupt_fn!(handle_irq120, 120);
gen_interrupt_fn!(handle_irq121, 121);
gen_interrupt_fn!(handle_irq122, 122);
gen_interrupt_fn!(handle_irq123, 123);
gen_interrupt_fn!(handle_irq124, 124);
gen_interrupt_fn!(handle_irq125, 125);
gen_interrupt_fn!(handle_irq126, 126);
gen_interrupt_fn!(handle_irq127, 127);
gen_interrupt_fn!(handle_irq128, 128);
gen_interrupt_fn!(handle_irq129, 129);
gen_interrupt_fn!(handle_irq130, 130);
gen_interrupt_fn!(handle_irq131, 131);
gen_interrupt_fn!(handle_irq132, 132);
gen_interrupt_fn!(handle_irq133, 133);
gen_interrupt_fn!(handle_irq134, 134);
gen_interrupt_fn!(handle_irq135, 135);
gen_interrupt_fn!(handle_irq136, 136);
gen_interrupt_fn!(handle_irq137, 137);
gen_interrupt_fn!(handle_irq138, 138);
gen_interrupt_fn!(handle_irq139, 139);
gen_interrupt_fn!(handle_irq140, 140);
gen_interrupt_fn!(handle_irq141, 141);
gen_interrupt_fn!(handle_irq142, 142);
gen_interrupt_fn!(handle_irq143, 143);
gen_interrupt_fn!(handle_irq144, 144);
gen_interrupt_fn!(handle_irq145, 145);
gen_interrupt_fn!(handle_irq146, 146);
gen_interrupt_fn!(handle_irq147, 147);
gen_interrupt_fn!(handle_irq148, 148);
gen_interrupt_fn!(handle_irq149, 149);
gen_interrupt_fn!(handle_irq150, 150);
gen_interrupt_fn!(handle_irq151, 151);
gen_interrupt_fn!(handle_irq152, 152);
gen_interrupt_fn!(handle_irq153, 153);
gen_interrupt_fn!(handle_irq154, 154);
gen_interrupt_fn!(handle_irq155, 155);
gen_interrupt_fn!(handle_irq156, 156);
gen_interrupt_fn!(handle_irq157, 157);
gen_interrupt_fn!(handle_irq158, 158);
gen_interrupt_fn!(handle_irq159, 159);
gen_interrupt_fn!(handle_irq160, 160);
gen_interrupt_fn!(handle_irq161, 161);
gen_interrupt_fn!(handle_irq162, 162);
gen_interrupt_fn!(handle_irq163, 163);
gen_interrupt_fn!(handle_irq164, 164);
gen_interrupt_fn!(handle_irq165, 165);
gen_interrupt_fn!(handle_irq166, 166);
gen_interrupt_fn!(handle_irq167, 167);
gen_interrupt_fn!(handle_irq168, 168);
gen_interrupt_fn!(handle_irq169, 169);
gen_interrupt_fn!(handle_irq170, 170);
gen_interrupt_fn!(handle_irq171, 171);
gen_interrupt_fn!(handle_irq172, 172);
gen_interrupt_fn!(handle_irq173, 173);
gen_interrupt_fn!(handle_irq174, 174);
gen_interrupt_fn!(handle_irq175, 175);
gen_interrupt_fn!(handle_irq176, 176);
gen_interrupt_fn!(handle_irq177, 177);
gen_interrupt_fn!(handle_irq178, 178);
gen_interrupt_fn!(handle_irq179, 179);
gen_interrupt_fn!(handle_irq180, 180);
gen_interrupt_fn!(handle_irq181, 181);
gen_interrupt_fn!(handle_irq182, 182);
gen_interrupt_fn!(handle_irq183, 183);
gen_interrupt_fn!(handle_irq184, 184);
gen_interrupt_fn!(handle_irq185, 185);
gen_interrupt_fn!(handle_irq186, 186);
gen_interrupt_fn!(handle_irq187, 187);
gen_interrupt_fn!(handle_irq188, 188);
gen_interrupt_fn!(handle_irq189, 189);
gen_interrupt_fn!(handle_irq190, 190);
gen_interrupt_fn!(handle_irq191, 191);
gen_interrupt_fn!(handle_irq192, 192);
gen_interrupt_fn!(handle_irq193, 193);
gen_interrupt_fn!(handle_irq194, 194);
gen_interrupt_fn!(handle_irq195, 195);
gen_interrupt_fn!(handle_irq196, 196);
gen_interrupt_fn!(handle_irq197, 197);
gen_interrupt_fn!(handle_irq198, 198);
gen_interrupt_fn!(handle_irq199, 199);
gen_interrupt_fn!(handle_irq200, 200);
gen_interrupt_fn!(handle_irq201, 201);
gen_interrupt_fn!(handle_irq202, 202);
gen_interrupt_fn!(handle_irq203, 203);
gen_interrupt_fn!(handle_irq204, 204);
gen_interrupt_fn!(handle_irq205, 205);
gen_interrupt_fn!(handle_irq206, 206);
gen_interrupt_fn!(handle_irq207, 207);
gen_interrupt_fn!(handle_irq208, 208);
gen_interrupt_fn!(handle_irq209, 209);
gen_interrupt_fn!(handle_irq210, 210);
gen_interrupt_fn!(handle_irq211, 211);
gen_interrupt_fn!(handle_irq212, 212);
gen_interrupt_fn!(handle_irq213, 213);
gen_interrupt_fn!(handle_irq214, 214);
gen_interrupt_fn!(handle_irq215, 215);
gen_interrupt_fn!(handle_irq216, 216);
gen_interrupt_fn!(handle_irq217, 217);
gen_interrupt_fn!(handle_irq218, 218);
gen_interrupt_fn!(handle_irq219, 219);
gen_interrupt_fn!(handle_irq220, 220);
gen_interrupt_fn!(handle_irq221, 221);
gen_interrupt_fn!(handle_irq222, 222);
gen_interrupt_fn!(handle_irq223, 223);
gen_interrupt_fn!(handle_irq224, 224);
gen_interrupt_fn!(handle_irq225, 225);
gen_interrupt_fn!(handle_irq226, 226);
gen_interrupt_fn!(handle_irq227, 227);
gen_interrupt_fn!(handle_irq228, 228);
gen_interrupt_fn!(handle_irq229, 229);
gen_interrupt_fn!(handle_irq230, 230);
gen_interrupt_fn!(handle_irq231, 231);
gen_interrupt_fn!(handle_irq232, 232);
gen_interrupt_fn!(handle_irq233, 233);
gen_interrupt_fn!(handle_irq234, 234);
gen_interrupt_fn!(handle_irq235, 235);
gen_interrupt_fn!(handle_irq236, 236);
gen_interrupt_fn!(handle_irq237, 237);
gen_interrupt_fn!(handle_irq238, 238);
gen_interrupt_fn!(handle_irq239, 239);
gen_interrupt_fn!(handle_irq240, 240);
gen_interrupt_fn!(handle_irq241, 241);
gen_interrupt_fn!(handle_irq242, 242);
gen_interrupt_fn!(handle_irq243, 243);
gen_interrupt_fn!(handle_irq244, 244);
gen_interrupt_fn!(handle_irq245, 245);
gen_interrupt_fn!(handle_irq246, 246);
gen_interrupt_fn!(handle_irq247, 247);
gen_interrupt_fn!(handle_irq248, 248);
gen_interrupt_fn!(handle_irq249, 249);
gen_interrupt_fn!(handle_irq250, 250);
gen_interrupt_fn!(handle_irq251, 251);
gen_interrupt_fn!(handle_irq252, 252);
gen_interrupt_fn!(handle_irq253, 253);
gen_interrupt_fn!(handle_irq254, 254);
gen_interrupt_fn!(handle_irq255, 255);

fn is_apic_available() -> bool {
    let apic_available_in_msr = {
        let apicbase = Msr::new(0x1B);
        unsafe { apicbase.read().get_bit(11) }
    };
    apic_available_in_msr && cpuid!(1).ecx.get_bit(9)
}

fn apic_addr() -> u64 {
    let apicbase = Msr::new(0x1B);
    unsafe { apicbase.read().get_bits(12..52) }
}

#[inline]
fn signal_eoi() {
    let mut eoi = Msr::new(0x80B);
    unsafe {
        eoi.write(0);
    }
}

/// Gets the tick count that has passed since we started counting
pub fn get_tick_count() -> u64 {
    TICK_COUNT.load(Ordering::SeqCst)
}

/// Sleeps for the given duration of nanoseconds.
pub fn sleep_for(duration: u64) {
    let hpet_info = get_hpet_info().unwrap();
    let intsts: VolAddress<u64, Safe, Safe> =
        unsafe { VolAddress::new(hpet_info.base_address + 0x20) };
    if duration < (hpet_info.clock_tick_unit as u64) {
        warn!(
            "Duration {} is less than minimum clock tick duration for HPET of {}; adjusting delay to {}",
            duration,
            hpet_info.clock_tick_unit,
            (hpet_info.clock_tick_unit as u64) + duration
        );
    }
    // Set HPET timer 0 in non-periodic mode
    let t0cfg: VolAddress<usize, Safe, Safe> =
        unsafe { VolAddress::new(hpet_info.base_address + (0x20 * 0) + 0x100) };
    let mut cfg = t0cfg.read();
    let oldcfg = cfg;
    if cfg.get_bit(4) {
        cfg.set_bit(3, true);
    }
    cfg.set_bit(2, true);
    cfg.set_bit(1, true);
    cfg.set_bit(8, false);
    t0cfg.write(cfg);
    let t0comp: VolAddress<u64, Safe, Safe> =
        unsafe { VolAddress::new(hpet_info.base_address + (0x20 * 0) + 0x108) };
    let duration = if duration < (hpet_info.clock_tick_unit as u64) {
        t0comp.read() + (hpet_info.clock_tick_unit as u64) + duration
    } else {
        t0comp.read() + duration
    };
    t0comp.write(duration);
    loop {
        if intsts.read().get_bit(0) {
            let mut int = intsts.read();
            int.set_bit(0, true);
            intsts.write(int);
            break;
        }
        hlt();
    }
    // Restore original timer configuration
    t0cfg.write(oldcfg);
}

/// Registers the given interrupt handler at the given interrupt. Note that this must be an interrupt
/// greater than or equal to 32.
pub fn register_interrupt_handler(interrupt: u8, func: InterruptHandler) -> usize {
    x86_64::instructions::interrupts::disable();
    debug!("Registering handler for int. {:X} ({:p})", interrupt, &func);
    let mut idx = 0usize;
    let mut tables = IRQ_FUNCS.write();
    if let Some(funcs) = tables.get_mut(&interrupt) {
        funcs.push(func);
        idx = funcs.len();
    }
    x86_64::instructions::interrupts::enable();
    idx
}

/// Unregisters the given interrupt handler given an interrupt number and function index ID.
pub fn unregister_interrupt_handler(int: u8, id: usize) -> bool {
    x86_64::instructions::interrupts::disable();
    debug!("Unregistering handler for int. {:X} (id {:X})", int, id);
    let irq = 32_u8.saturating_add(int);
    if let Some(funcs) = IRQ_FUNCS.write().get_mut(&irq) {
        if funcs.len() >= id {
            let _ = funcs.remove(id);
        } else {
            x86_64::instructions::interrupts::enable();
            return false;
        }
    }
    x86_64::instructions::interrupts::enable();
    true
}
