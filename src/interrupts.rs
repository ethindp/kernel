use crate::gdt;
use cpuio::{inb, outb};
use heapless::consts::*;
use heapless::{FnvIndexMap, Vec};
use lazy_static::lazy_static;
use raw_cpuid::*;
use spin::RwLock;
use x86_64::registers::model_specific::Msr;
use x86_64::{
    structures::idt::PageFaultErrorCode,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

/// This enumeration contains a list of all IRQs.
#[repr(u8)]
pub enum InterruptType {
    Timer = 32, // IRQ 0 - system timer (cannot be changed)
    Keyboard,   // IRQ 1 - keyboard controller (cannot be changed)
    Cascade, // IRQ 2 - cascaded signals from IRQs 8-15 (any devices configured to use IRQ 2 will actually be using IRQ 9)
    Uart1, // IRQ 3 - serial port controller for serial port 2 (shared with serial port 4, if present)
    Serial1, // IRQ 4 - serial port controller for serial port 1 (shared with serial port 3, if present)
    Parallel, // IRQ 5 - parallel port 2 and 3  or  sound card
    Floppy,  // IRQ 6 - floppy disk controller
    Lpt1, // IRQ 7 - parallel port 1. It is used for printers or for any parallel port if a printer is not present. It can also be potentially be shared with a secondary sound card with careful management of the port.
    Rtc,  // IRQ 8 - real-time clock (RTC)
    Acpi, // IRQ 9 - Advanced Configuration and Power Interface (ACPI) system control interrupt on Intel chipsets.
    // Other chipset manufacturers might use another interrupt for this purpose, or make it available for the use of peripherals (any devices configured to use IRQ 2 will actually be using IRQ 9)
    Open1, // IRQ 10 - The Interrupt is left open for the use of peripherals (open interrupt/available, SCSI or NIC)
    Open2, // IRQ 11 - The Interrupt is left open for the use of peripherals (open interrupt/available, SCSI or NIC)
    Mouse, // IRQ 12 - mouse on PS/2 connector
    Coprocessor, // IRQ 13 - CPU co-processor  or  integrated floating point unit  or  inter-processor interrupt (use depends on OS)
    PrimaryAta, // IRQ 14 - primary ATA channel (ATA interface usually serves hard disk drives and CD drives)
    SecondaryAta, // IRQ 15 - secondary ATA channel
    // Other interrupts
    Irq48 = 48,
    Irq49,
    Irq50,
    Irq51,
    Irq52,
    Irq53,
    Irq54,
    Irq55,
    Irq56,
    Irq57,
    Irq58,
    Irq59,
    Irq60,
    Irq61,
    Irq62,
    Irq63,
    Irq64,
    Irq65,
    Irq66,
    Irq67,
    Irq68,
    Irq69,
    Irq70,
    Irq71,
    Irq72,
    Irq73,
    Irq74,
    Irq75,
    Irq76,
    Irq77,
    Irq78,
    Irq79,
    Irq80,
    Irq81,
    Irq82,
    Irq83,
    Irq84,
    Irq85,
    Irq86,
    Irq87,
    Irq88,
    Irq89,
    Irq90,
    Irq91,
    Irq92,
    Irq93,
    Irq94,
    Irq95,
    Irq96,
    Irq97,
    Irq98,
    Irq99,
    Irq100,
    Irq101,
    Irq102,
    Irq103,
    Irq104,
    Irq105,
    Irq106,
    Irq107,
    Irq108,
    Irq109,
    Irq110,
    Irq111,
    Irq112,
    Irq113,
    Irq114,
    Irq115,
    Irq116,
    Irq117,
    Irq118,
    Irq119,
    Irq120,
    Irq121,
    Irq122,
    Irq123,
    Irq124,
    Irq125,
    Irq126,
    Irq127,
    Irq128,
    Irq129,
    Irq130,
    Irq131,
    Irq132,
    Irq133,
    Irq134,
    Irq135,
    Irq136,
    Irq137,
    Irq138,
    Irq139,
    Irq140,
    Irq141,
    Irq142,
    Irq143,
    Irq144,
    Irq145,
    Irq146,
    Irq147,
    Irq148,
    Irq149,
    Irq150,
    Irq151,
    Irq152,
    Irq153,
    Irq154,
    Irq155,
    Irq156,
    Irq157,
    Irq158,
    Irq159,
    Irq160,
    Irq161,
    Irq162,
    Irq163,
    Irq164,
    Irq165,
    Irq166,
    Irq167,
    Irq168,
    Irq169,
    Irq170,
    Irq171,
    Irq172,
    Irq173,
    Irq174,
    Irq175,
    Irq176,
    Irq177,
    Irq178,
    Irq179,
    Irq180,
    Irq181,
    Irq182,
    Irq183,
    Irq184,
    Irq185,
    Irq186,
    Irq187,
    Irq188,
    Irq189,
    Irq190,
    Irq191,
    Irq192,
    Irq193,
    Irq194,
    Irq195,
    Irq196,
    Irq197,
    Irq198,
    Irq199,
    Irq200,
    Irq201,
    Irq202,
    Irq203,
    Irq204,
    Irq205,
    Irq206,
    Irq207,
    Irq208,
    Irq209,
    Irq210,
    Irq211,
    Irq212,
    Irq213,
    Irq214,
    Irq215,
    Irq216,
    Irq217,
    Irq218,
    Irq219,
    Irq220,
    Irq221,
    Irq222,
    Irq223,
    Irq224,
    Irq225,
    Irq226,
    Irq227,
    Irq228,
    Irq229,
    Irq230,
    Irq231,
    Irq232,
    Irq233,
    Irq234,
    Irq235,
    Irq236,
    Irq237,
    Irq238,
    Irq239,
    Irq240,
    Irq241,
    Irq242,
    Irq243,
    Irq244,
    Irq245,
    Irq246,
    Irq247,
    Irq248,
    Irq249,
    Irq250,
    Irq251,
    Irq252,
    Irq253,
    Irq254,
    Irq255,
}

impl InterruptType {
    fn convert_to_u8(self) -> u8 {
        self as u8
    }

    fn convert_to_usize(self) -> usize {
        usize::from(self.convert_to_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
// Handle BPs
        idt.breakpoint.set_handler_fn(handle_bp);
// Handle DFs (on our set-up separate 4K kernel stack)
        unsafe {
            idt.double_fault
                .set_handler_fn(handle_df)
                .set_stack_index(gdt::DF_IST_IDX);
        }
        idt.page_fault.set_handler_fn(handle_pf);
        idt.overflow.set_handler_fn(handle_of);
        idt.bound_range_exceeded.set_handler_fn(handle_br);
        idt.invalid_opcode.set_handler_fn(handle_ud);
        idt.device_not_available.set_handler_fn(handle_nm);
        idt.general_protection_fault.set_handler_fn(handle_gp);
idt[InterruptType::Timer.convert_to_usize()].set_handler_fn(handle_timer);
idt[InterruptType::Keyboard.convert_to_usize()].set_handler_fn(handle_keyboard);
idt[InterruptType::Cascade.convert_to_usize()].set_handler_fn(handle_cascade);
idt[InterruptType::Uart1.convert_to_usize()].set_handler_fn(handle_uart1);
idt[InterruptType::Serial1.convert_to_usize()].set_handler_fn(handle_serial1);
idt[InterruptType::Parallel.convert_to_usize()].set_handler_fn(handle_parallel);
idt[InterruptType::Floppy.convert_to_usize()].set_handler_fn(handle_floppy);
idt[InterruptType::Lpt1.convert_to_usize()].set_handler_fn(handle_lpt1);
idt[InterruptType::Rtc.convert_to_usize()].set_handler_fn(handle_rtc);
idt[InterruptType::Acpi.convert_to_usize()].set_handler_fn(handle_acpi);
idt[InterruptType::Open1.convert_to_usize()].set_handler_fn(handle_open1);
idt[InterruptType::Open2.convert_to_usize()].set_handler_fn(handle_open2);
idt[InterruptType::Mouse.convert_to_usize()].set_handler_fn(handle_mouse);
idt[InterruptType::Coprocessor.convert_to_usize()].set_handler_fn(handle_coprocessor);
idt[InterruptType::PrimaryAta.convert_to_usize()].set_handler_fn(handle_primary_ata);
idt[InterruptType::SecondaryAta.convert_to_usize()].set_handler_fn(handle_secondary_ata);
idt[InterruptType::Irq48.convert_to_usize()].set_handler_fn(handle_irq48);
idt[InterruptType::Irq49.convert_to_usize()].set_handler_fn(handle_irq49);
idt[InterruptType::Irq50.convert_to_usize()].set_handler_fn(handle_irq50);
idt[InterruptType::Irq51.convert_to_usize()].set_handler_fn(handle_irq51);
idt[InterruptType::Irq52.convert_to_usize()].set_handler_fn(handle_irq52);
idt[InterruptType::Irq53.convert_to_usize()].set_handler_fn(handle_irq53);
idt[InterruptType::Irq54.convert_to_usize()].set_handler_fn(handle_irq54);
idt[InterruptType::Irq55.convert_to_usize()].set_handler_fn(handle_irq55);
idt[InterruptType::Irq56.convert_to_usize()].set_handler_fn(handle_irq56);
idt[InterruptType::Irq57.convert_to_usize()].set_handler_fn(handle_irq57);
idt[InterruptType::Irq58.convert_to_usize()].set_handler_fn(handle_irq58);
idt[InterruptType::Irq59.convert_to_usize()].set_handler_fn(handle_irq59);
idt[InterruptType::Irq60.convert_to_usize()].set_handler_fn(handle_irq60);
idt[InterruptType::Irq61.convert_to_usize()].set_handler_fn(handle_irq61);
idt[InterruptType::Irq62.convert_to_usize()].set_handler_fn(handle_irq62);
idt[InterruptType::Irq63.convert_to_usize()].set_handler_fn(handle_irq63);
idt[InterruptType::Irq64.convert_to_usize()].set_handler_fn(handle_irq64);
idt[InterruptType::Irq65.convert_to_usize()].set_handler_fn(handle_irq65);
idt[InterruptType::Irq66.convert_to_usize()].set_handler_fn(handle_irq66);
idt[InterruptType::Irq67.convert_to_usize()].set_handler_fn(handle_irq67);
idt[InterruptType::Irq68.convert_to_usize()].set_handler_fn(handle_irq68);
idt[InterruptType::Irq69.convert_to_usize()].set_handler_fn(handle_irq69);
idt[InterruptType::Irq70.convert_to_usize()].set_handler_fn(handle_irq70);
idt[InterruptType::Irq71.convert_to_usize()].set_handler_fn(handle_irq71);
idt[InterruptType::Irq72.convert_to_usize()].set_handler_fn(handle_irq72);
idt[InterruptType::Irq73.convert_to_usize()].set_handler_fn(handle_irq73);
idt[InterruptType::Irq74.convert_to_usize()].set_handler_fn(handle_irq74);
idt[InterruptType::Irq75.convert_to_usize()].set_handler_fn(handle_irq75);
idt[InterruptType::Irq76.convert_to_usize()].set_handler_fn(handle_irq76);
idt[InterruptType::Irq77.convert_to_usize()].set_handler_fn(handle_irq77);
idt[InterruptType::Irq78.convert_to_usize()].set_handler_fn(handle_irq78);
idt[InterruptType::Irq79.convert_to_usize()].set_handler_fn(handle_irq79);
idt[InterruptType::Irq80.convert_to_usize()].set_handler_fn(handle_irq80);
idt[InterruptType::Irq81.convert_to_usize()].set_handler_fn(handle_irq81);
idt[InterruptType::Irq82.convert_to_usize()].set_handler_fn(handle_irq82);
idt[InterruptType::Irq83.convert_to_usize()].set_handler_fn(handle_irq83);
idt[InterruptType::Irq84.convert_to_usize()].set_handler_fn(handle_irq84);
idt[InterruptType::Irq85.convert_to_usize()].set_handler_fn(handle_irq85);
idt[InterruptType::Irq86.convert_to_usize()].set_handler_fn(handle_irq86);
idt[InterruptType::Irq87.convert_to_usize()].set_handler_fn(handle_irq87);
idt[InterruptType::Irq88.convert_to_usize()].set_handler_fn(handle_irq88);
idt[InterruptType::Irq89.convert_to_usize()].set_handler_fn(handle_irq89);
idt[InterruptType::Irq90.convert_to_usize()].set_handler_fn(handle_irq90);
idt[InterruptType::Irq91.convert_to_usize()].set_handler_fn(handle_irq91);
idt[InterruptType::Irq92.convert_to_usize()].set_handler_fn(handle_irq92);
idt[InterruptType::Irq93.convert_to_usize()].set_handler_fn(handle_irq93);
idt[InterruptType::Irq94.convert_to_usize()].set_handler_fn(handle_irq94);
idt[InterruptType::Irq95.convert_to_usize()].set_handler_fn(handle_irq95);
idt[InterruptType::Irq96.convert_to_usize()].set_handler_fn(handle_irq96);
idt[InterruptType::Irq97.convert_to_usize()].set_handler_fn(handle_irq97);
idt[InterruptType::Irq98.convert_to_usize()].set_handler_fn(handle_irq98);
idt[InterruptType::Irq99.convert_to_usize()].set_handler_fn(handle_irq99);
idt[InterruptType::Irq100.convert_to_usize()].set_handler_fn(handle_irq100);
idt[InterruptType::Irq101.convert_to_usize()].set_handler_fn(handle_irq101);
idt[InterruptType::Irq102.convert_to_usize()].set_handler_fn(handle_irq102);
idt[InterruptType::Irq103.convert_to_usize()].set_handler_fn(handle_irq103);
idt[InterruptType::Irq104.convert_to_usize()].set_handler_fn(handle_irq104);
idt[InterruptType::Irq105.convert_to_usize()].set_handler_fn(handle_irq105);
idt[InterruptType::Irq106.convert_to_usize()].set_handler_fn(handle_irq106);
idt[InterruptType::Irq107.convert_to_usize()].set_handler_fn(handle_irq107);
idt[InterruptType::Irq108.convert_to_usize()].set_handler_fn(handle_irq108);
idt[InterruptType::Irq109.convert_to_usize()].set_handler_fn(handle_irq109);
idt[InterruptType::Irq110.convert_to_usize()].set_handler_fn(handle_irq110);
idt[InterruptType::Irq111.convert_to_usize()].set_handler_fn(handle_irq111);
idt[InterruptType::Irq112.convert_to_usize()].set_handler_fn(handle_irq112);
idt[InterruptType::Irq113.convert_to_usize()].set_handler_fn(handle_irq113);
idt[InterruptType::Irq114.convert_to_usize()].set_handler_fn(handle_irq114);
idt[InterruptType::Irq115.convert_to_usize()].set_handler_fn(handle_irq115);
idt[InterruptType::Irq116.convert_to_usize()].set_handler_fn(handle_irq116);
idt[InterruptType::Irq117.convert_to_usize()].set_handler_fn(handle_irq117);
idt[InterruptType::Irq118.convert_to_usize()].set_handler_fn(handle_irq118);
idt[InterruptType::Irq119.convert_to_usize()].set_handler_fn(handle_irq119);
idt[InterruptType::Irq120.convert_to_usize()].set_handler_fn(handle_irq120);
idt[InterruptType::Irq121.convert_to_usize()].set_handler_fn(handle_irq121);
idt[InterruptType::Irq122.convert_to_usize()].set_handler_fn(handle_irq122);
idt[InterruptType::Irq123.convert_to_usize()].set_handler_fn(handle_irq123);
idt[InterruptType::Irq124.convert_to_usize()].set_handler_fn(handle_irq124);
idt[InterruptType::Irq125.convert_to_usize()].set_handler_fn(handle_irq125);
idt[InterruptType::Irq126.convert_to_usize()].set_handler_fn(handle_irq126);
idt[InterruptType::Irq127.convert_to_usize()].set_handler_fn(handle_irq127);
idt[InterruptType::Irq128.convert_to_usize()].set_handler_fn(handle_irq128);
idt[InterruptType::Irq129.convert_to_usize()].set_handler_fn(handle_irq129);
idt[InterruptType::Irq130.convert_to_usize()].set_handler_fn(handle_irq130);
idt[InterruptType::Irq131.convert_to_usize()].set_handler_fn(handle_irq131);
idt[InterruptType::Irq132.convert_to_usize()].set_handler_fn(handle_irq132);
idt[InterruptType::Irq133.convert_to_usize()].set_handler_fn(handle_irq133);
idt[InterruptType::Irq134.convert_to_usize()].set_handler_fn(handle_irq134);
idt[InterruptType::Irq135.convert_to_usize()].set_handler_fn(handle_irq135);
idt[InterruptType::Irq136.convert_to_usize()].set_handler_fn(handle_irq136);
idt[InterruptType::Irq137.convert_to_usize()].set_handler_fn(handle_irq137);
idt[InterruptType::Irq138.convert_to_usize()].set_handler_fn(handle_irq138);
idt[InterruptType::Irq139.convert_to_usize()].set_handler_fn(handle_irq139);
idt[InterruptType::Irq140.convert_to_usize()].set_handler_fn(handle_irq140);
idt[InterruptType::Irq141.convert_to_usize()].set_handler_fn(handle_irq141);
idt[InterruptType::Irq142.convert_to_usize()].set_handler_fn(handle_irq142);
idt[InterruptType::Irq143.convert_to_usize()].set_handler_fn(handle_irq143);
idt[InterruptType::Irq144.convert_to_usize()].set_handler_fn(handle_irq144);
idt[InterruptType::Irq145.convert_to_usize()].set_handler_fn(handle_irq145);
idt[InterruptType::Irq146.convert_to_usize()].set_handler_fn(handle_irq146);
idt[InterruptType::Irq147.convert_to_usize()].set_handler_fn(handle_irq147);
idt[InterruptType::Irq148.convert_to_usize()].set_handler_fn(handle_irq148);
idt[InterruptType::Irq149.convert_to_usize()].set_handler_fn(handle_irq149);
idt[InterruptType::Irq150.convert_to_usize()].set_handler_fn(handle_irq150);
idt[InterruptType::Irq151.convert_to_usize()].set_handler_fn(handle_irq151);
idt[InterruptType::Irq152.convert_to_usize()].set_handler_fn(handle_irq152);
idt[InterruptType::Irq153.convert_to_usize()].set_handler_fn(handle_irq153);
idt[InterruptType::Irq154.convert_to_usize()].set_handler_fn(handle_irq154);
idt[InterruptType::Irq155.convert_to_usize()].set_handler_fn(handle_irq155);
idt[InterruptType::Irq156.convert_to_usize()].set_handler_fn(handle_irq156);
idt[InterruptType::Irq157.convert_to_usize()].set_handler_fn(handle_irq157);
idt[InterruptType::Irq158.convert_to_usize()].set_handler_fn(handle_irq158);
idt[InterruptType::Irq159.convert_to_usize()].set_handler_fn(handle_irq159);
idt[InterruptType::Irq160.convert_to_usize()].set_handler_fn(handle_irq160);
idt[InterruptType::Irq161.convert_to_usize()].set_handler_fn(handle_irq161);
idt[InterruptType::Irq162.convert_to_usize()].set_handler_fn(handle_irq162);
idt[InterruptType::Irq163.convert_to_usize()].set_handler_fn(handle_irq163);
idt[InterruptType::Irq164.convert_to_usize()].set_handler_fn(handle_irq164);
idt[InterruptType::Irq165.convert_to_usize()].set_handler_fn(handle_irq165);
idt[InterruptType::Irq166.convert_to_usize()].set_handler_fn(handle_irq166);
idt[InterruptType::Irq167.convert_to_usize()].set_handler_fn(handle_irq167);
idt[InterruptType::Irq168.convert_to_usize()].set_handler_fn(handle_irq168);
idt[InterruptType::Irq169.convert_to_usize()].set_handler_fn(handle_irq169);
idt[InterruptType::Irq170.convert_to_usize()].set_handler_fn(handle_irq170);
idt[InterruptType::Irq171.convert_to_usize()].set_handler_fn(handle_irq171);
idt[InterruptType::Irq172.convert_to_usize()].set_handler_fn(handle_irq172);
idt[InterruptType::Irq173.convert_to_usize()].set_handler_fn(handle_irq173);
idt[InterruptType::Irq174.convert_to_usize()].set_handler_fn(handle_irq174);
idt[InterruptType::Irq175.convert_to_usize()].set_handler_fn(handle_irq175);
idt[InterruptType::Irq176.convert_to_usize()].set_handler_fn(handle_irq176);
idt[InterruptType::Irq177.convert_to_usize()].set_handler_fn(handle_irq177);
idt[InterruptType::Irq178.convert_to_usize()].set_handler_fn(handle_irq178);
idt[InterruptType::Irq179.convert_to_usize()].set_handler_fn(handle_irq179);
idt[InterruptType::Irq180.convert_to_usize()].set_handler_fn(handle_irq180);
idt[InterruptType::Irq181.convert_to_usize()].set_handler_fn(handle_irq181);
idt[InterruptType::Irq182.convert_to_usize()].set_handler_fn(handle_irq182);
idt[InterruptType::Irq183.convert_to_usize()].set_handler_fn(handle_irq183);
idt[InterruptType::Irq184.convert_to_usize()].set_handler_fn(handle_irq184);
idt[InterruptType::Irq185.convert_to_usize()].set_handler_fn(handle_irq185);
idt[InterruptType::Irq186.convert_to_usize()].set_handler_fn(handle_irq186);
idt[InterruptType::Irq187.convert_to_usize()].set_handler_fn(handle_irq187);
idt[InterruptType::Irq188.convert_to_usize()].set_handler_fn(handle_irq188);
idt[InterruptType::Irq189.convert_to_usize()].set_handler_fn(handle_irq189);
idt[InterruptType::Irq190.convert_to_usize()].set_handler_fn(handle_irq190);
idt[InterruptType::Irq191.convert_to_usize()].set_handler_fn(handle_irq191);
idt[InterruptType::Irq192.convert_to_usize()].set_handler_fn(handle_irq192);
idt[InterruptType::Irq193.convert_to_usize()].set_handler_fn(handle_irq193);
idt[InterruptType::Irq194.convert_to_usize()].set_handler_fn(handle_irq194);
idt[InterruptType::Irq195.convert_to_usize()].set_handler_fn(handle_irq195);
idt[InterruptType::Irq196.convert_to_usize()].set_handler_fn(handle_irq196);
idt[InterruptType::Irq197.convert_to_usize()].set_handler_fn(handle_irq197);
idt[InterruptType::Irq198.convert_to_usize()].set_handler_fn(handle_irq198);
idt[InterruptType::Irq199.convert_to_usize()].set_handler_fn(handle_irq199);
idt[InterruptType::Irq200.convert_to_usize()].set_handler_fn(handle_irq200);
idt[InterruptType::Irq201.convert_to_usize()].set_handler_fn(handle_irq201);
idt[InterruptType::Irq202.convert_to_usize()].set_handler_fn(handle_irq202);
idt[InterruptType::Irq203.convert_to_usize()].set_handler_fn(handle_irq203);
idt[InterruptType::Irq204.convert_to_usize()].set_handler_fn(handle_irq204);
idt[InterruptType::Irq205.convert_to_usize()].set_handler_fn(handle_irq205);
idt[InterruptType::Irq206.convert_to_usize()].set_handler_fn(handle_irq206);
idt[InterruptType::Irq207.convert_to_usize()].set_handler_fn(handle_irq207);
idt[InterruptType::Irq208.convert_to_usize()].set_handler_fn(handle_irq208);
idt[InterruptType::Irq209.convert_to_usize()].set_handler_fn(handle_irq209);
idt[InterruptType::Irq210.convert_to_usize()].set_handler_fn(handle_irq210);
idt[InterruptType::Irq211.convert_to_usize()].set_handler_fn(handle_irq211);
idt[InterruptType::Irq212.convert_to_usize()].set_handler_fn(handle_irq212);
idt[InterruptType::Irq213.convert_to_usize()].set_handler_fn(handle_irq213);
idt[InterruptType::Irq214.convert_to_usize()].set_handler_fn(handle_irq214);
idt[InterruptType::Irq215.convert_to_usize()].set_handler_fn(handle_irq215);
idt[InterruptType::Irq216.convert_to_usize()].set_handler_fn(handle_irq216);
idt[InterruptType::Irq217.convert_to_usize()].set_handler_fn(handle_irq217);
idt[InterruptType::Irq218.convert_to_usize()].set_handler_fn(handle_irq218);
idt[InterruptType::Irq219.convert_to_usize()].set_handler_fn(handle_irq219);
idt[InterruptType::Irq220.convert_to_usize()].set_handler_fn(handle_irq220);
idt[InterruptType::Irq221.convert_to_usize()].set_handler_fn(handle_irq221);
idt[InterruptType::Irq222.convert_to_usize()].set_handler_fn(handle_irq222);
idt[InterruptType::Irq223.convert_to_usize()].set_handler_fn(handle_irq223);
idt[InterruptType::Irq224.convert_to_usize()].set_handler_fn(handle_irq224);
idt[InterruptType::Irq225.convert_to_usize()].set_handler_fn(handle_irq225);
idt[InterruptType::Irq226.convert_to_usize()].set_handler_fn(handle_irq226);
idt[InterruptType::Irq227.convert_to_usize()].set_handler_fn(handle_irq227);
idt[InterruptType::Irq228.convert_to_usize()].set_handler_fn(handle_irq228);
idt[InterruptType::Irq229.convert_to_usize()].set_handler_fn(handle_irq229);
idt[InterruptType::Irq230.convert_to_usize()].set_handler_fn(handle_irq230);
idt[InterruptType::Irq231.convert_to_usize()].set_handler_fn(handle_irq231);
idt[InterruptType::Irq232.convert_to_usize()].set_handler_fn(handle_irq232);
idt[InterruptType::Irq233.convert_to_usize()].set_handler_fn(handle_irq233);
idt[InterruptType::Irq234.convert_to_usize()].set_handler_fn(handle_irq234);
idt[InterruptType::Irq235.convert_to_usize()].set_handler_fn(handle_irq235);
idt[InterruptType::Irq236.convert_to_usize()].set_handler_fn(handle_irq236);
idt[InterruptType::Irq237.convert_to_usize()].set_handler_fn(handle_irq237);
idt[InterruptType::Irq238.convert_to_usize()].set_handler_fn(handle_irq238);
idt[InterruptType::Irq239.convert_to_usize()].set_handler_fn(handle_irq239);
idt[InterruptType::Irq240.convert_to_usize()].set_handler_fn(handle_irq240);
idt[InterruptType::Irq241.convert_to_usize()].set_handler_fn(handle_irq241);
idt[InterruptType::Irq242.convert_to_usize()].set_handler_fn(handle_irq242);
idt[InterruptType::Irq243.convert_to_usize()].set_handler_fn(handle_irq243);
idt[InterruptType::Irq244.convert_to_usize()].set_handler_fn(handle_irq244);
idt[InterruptType::Irq245.convert_to_usize()].set_handler_fn(handle_irq245);
idt[InterruptType::Irq246.convert_to_usize()].set_handler_fn(handle_irq246);
idt[InterruptType::Irq247.convert_to_usize()].set_handler_fn(handle_irq247);
idt[InterruptType::Irq248.convert_to_usize()].set_handler_fn(handle_irq248);
idt[InterruptType::Irq249.convert_to_usize()].set_handler_fn(handle_irq249);
idt[InterruptType::Irq250.convert_to_usize()].set_handler_fn(handle_irq250);
idt[InterruptType::Irq251.convert_to_usize()].set_handler_fn(handle_irq251);
idt[InterruptType::Irq252.convert_to_usize()].set_handler_fn(handle_irq252);
idt[InterruptType::Irq253.convert_to_usize()].set_handler_fn(handle_irq253);
idt[InterruptType::Irq254.convert_to_usize()].set_handler_fn(handle_irq254);
idt[InterruptType::Irq255.convert_to_usize()].set_handler_fn(handle_irq255);
        idt
    };
    static ref TICK_COUNT: RwLock<u128> = RwLock::new(0u128);
static ref IRQ_FUNCS: RwLock<FnvIndexMap<u8, Vec<fn(), U0>, U256>> = RwLock::new({
let mut table = FnvIndexMap::<u8, Vec<fn(), U0>, U256>::new();
for i in 0 .. 256 {
let v = Vec::<fn(), U0>::new();
table.insert(i as u8, v).unwrap();
}
table
});
}

pub fn init_stage1() {
    use crate::memory::allocate_phys_range;
    use crate::printkln;
    printkln!("INTR: Stage 1 initialization started");
    unsafe {
        printkln!("INTR: PIC: Acquiring masks");
        let saved_mask1 = inb(0x21);
        let saved_mask2 = inb(0xA1);
        printkln!("INTR: PIC: Masks: {:X}h, {:X}h", saved_mask1, saved_mask2);
        printkln!("INTR: PIC: Sending initialization command");
        outb(0x11, 0x20);
        outb(0, 0x80);
        outb(0x11, 0xA0);
        outb(0, 0x80);
        printkln!("INTR: PIC: Setting base offsets to 20h and 28h");
        outb(0x20, 0x21);
        outb(0, 0x80);
        outb(0x28, 0xA1);
        outb(0, 0x80);
        printkln!("INTR: PIC: Setting up chain for master and slave");
        outb(0x04, 0x21);
        outb(0, 0x80);
        outb(0x02, 0xA1);
        outb(0, 0x80);
        printkln!("INTR: PIC: Setting mode to 1h");
        outb(0x01, 0x21);
        outb(0, 0x80);
        outb(0x01, 0xA1);
        outb(0, 0x80);
        printkln!("INTR: PIC: Restoring PIC masks");
        outb(saved_mask1, 0x21);
        outb(0, 0x80);
        outb(saved_mask2, 0xA1);
        outb(0, 0x80);
        if is_apic_available() {
            allocate_phys_range(apic_addr(), apic_addr() + 0x530);
        }
    }
    printkln!("INTR: Loading IDT");
    IDT.load();
    printkln!("INTR: Enabling interrupts");
    x86_64::instructions::interrupts::enable();
    printkln!("INTR: Stage 1 initialization complete");
}

pub fn init_stage2() {
    use crate::printkln;
    x86_64::instructions::interrupts::disable();
    printkln!("INTR: loading IDT");
    IDT.load();
    if is_apic_available() {
        // Initialize PIC, then mask everything
        printkln!("INTR: APIC is available and usable; configuring");
        unsafe {
            let saved_mask1 = inb(0x21);
            let saved_mask2 = inb(0xA1);
            outb(0x11, 0x20);
            outb(0, 0x80);
            outb(0x11, 0xA0);
            outb(0, 0x80);
            outb(0x20, 0x21);
            outb(0, 0x80);
            outb(0x28, 0xA1);
            outb(0, 0x80);
            outb(0x04, 0x21);
            outb(0, 0x80);
            outb(0x02, 0xA1);
            outb(0, 0x80);
            outb(0x01, 0x21);
            outb(0, 0x80);
            outb(0x01, 0xA1);
            outb(0, 0x80);
            outb(saved_mask1, 0x21);
            outb(0, 0x80);
            outb(saved_mask2, 0xA1);
            outb(0, 0x80);
            outb(0xFF, 0xA1);
            outb(0, 0x80);
            outb(0xFF, 0x21);
            outb(0, 0x80);
        }
        let apic_reg = (apic_addr() + 0xF0) as *mut u32;
        unsafe {
            *(apic_reg) |= 0x100;
        }
        printkln!("INTR: APIC configuration complete");
    } else {
        printkln!("INTR: APIC not available/supported, falling back to 8259 PIC");
        printkln!("INTR: Configuring PIC");
        unsafe {
            let saved_mask1 = inb(0x21);
            let saved_mask2 = inb(0xA1);
            outb(0x11, 0x20);
            outb(0, 0x80);
            outb(0x11, 0xA0);
            outb(0, 0x80);
            outb(0x20, 0x21);
            outb(0, 0x80);
            outb(0x28, 0xA1);
            outb(0, 0x80);
            outb(0x04, 0x21);
            outb(0, 0x80);
            outb(0x02, 0xA1);
            outb(0, 0x80);
            outb(0x01, 0x21);
            outb(0, 0x80);
            outb(0x01, 0xA1);
            outb(0, 0x80);
            outb(saved_mask1, 0x21);
            outb(0, 0x80);
            outb(saved_mask2, 0xA1);
            outb(0, 0x80);
        }
        printkln!("INTR: PIC configuration complete");
    }
    x86_64::instructions::interrupts::enable();
}

// Macro to generate interrupt functions
macro_rules! gen_interrupt_fn {
    ($i:ident, $p:path) => {
        extern "x86-interrupt" fn $i(_stack_frame: &mut InterruptStackFrame) {
            if let Some(tbl) = IRQ_FUNCS.try_read() {
                for func in tbl.get(&$p.convert_to_u8()).unwrap().iter() {
                    (func)();
                }
            }
            signal_eoi($p.convert_to_u8());
        }
    };
}

extern "x86-interrupt" fn handle_bp(stack_frame: &mut InterruptStackFrame) {
    // All we do here is notify the user and continue on.
    use crate::printkln;
    printkln!(
        "Hardware breakpoint interrupt received:\n{:#?}",
        stack_frame
    );
}

extern "x86-interrupt" fn handle_df(stack_frame: &mut InterruptStackFrame, error_code: u64) -> ! {
    unsafe {
        llvm_asm!("push rax" :::: "intel");
    }
    panic!(
        "EXCEPTION: DOUBLE FAULT({})\n{:#?}",
        error_code, stack_frame,
    );
}

extern "x86-interrupt" fn handle_timer(_: &mut InterruptStackFrame) {
    if let Some(tbl) = IRQ_FUNCS.try_read() {
        for func in tbl
            .get(&InterruptType::Timer.convert_to_u8())
            .unwrap()
            .iter()
        {
            (func)();
        }
    }
    signal_eoi(InterruptType::Timer.convert_to_u8());
}

extern "x86-interrupt" fn handle_rtc(_stack_frame: &mut InterruptStackFrame) {
    if let Some(mut tc) = TICK_COUNT.try_write() {
        *tc += 1;
    }
    if let Some(tbl) = IRQ_FUNCS.try_read() {
        for func in tbl.get(&InterruptType::Rtc.convert_to_u8()).unwrap().iter() {
            (func)();
        }
    }
    signal_eoi(InterruptType::Rtc.convert_to_u8());
    unsafe {
        outb(0x0C, 0x70);
        inb(0x71);
    }
}

extern "x86-interrupt" fn handle_keyboard(_stack_frame: &mut InterruptStackFrame) {
    if let Some(tbl) = IRQ_FUNCS.try_read() {
        for func in tbl
            .get(&InterruptType::Keyboard.convert_to_u8())
            .unwrap()
            .iter()
        {
            (func)();
        }
    }
    signal_eoi(InterruptType::Keyboard.convert_to_u8());
}

extern "x86-interrupt" fn handle_pf(_: &mut InterruptStackFrame, error_code: PageFaultErrorCode) {
    unsafe {
        llvm_asm!("push rax" :::: "intel");
    }
    use crate::idle_forever;
    use crate::printkln;
    use x86_64::registers::control::Cr2;
    let addr = Cr2::read();
    let ec = error_code.bits();
    printkln!(
        "Page fault: {} while {} to memory address {:X}h",
        if (ec & 1) > 0 {
            "Protection violation"
        } else if !(ec & 1) > 0 {
            "Page not present"
        } else if (ec & 1 << 2) > 0 {
            "Possible privilege violation (user mode)"
        } else if !(ec & 1 << 2) > 0 {
            "Possible privilege violation (kernel mode)"
        } else if ec & 1 << 3 > 0 {
            "Attempted read of reserved PTT entry"
        } else if ec & 1 << 4 > 0 {
            "Instruction fetch"
        } else {
            "unknown cause"
        },
        if ec & 1 << 1 > 0 {
            "writing"
        } else {
            "reading"
        },
        addr.as_u64()
    );
    idle_forever();
}

extern "x86-interrupt" fn handle_of(_: &mut InterruptStackFrame) {
    use crate::printkln;
    printkln!("Warning: can't execute calculation: overflow");
}

extern "x86-interrupt" fn handle_br(stack: &mut InterruptStackFrame) {
    panic!(
        "Cannot continue: bounds range exceeded.\nStack:\n{:?}",
        stack,
    );
}

extern "x86-interrupt" fn handle_ud(stack: &mut InterruptStackFrame) {
    panic!("Cannot continue: invalid opcode!\nStack:\n{:?}", stack,);
}

extern "x86-interrupt" fn handle_nm(stack: &mut InterruptStackFrame) {
    panic!("Can't continue: device unavailable!\nStack:\n{:?}", stack,);
}

extern "x86-interrupt" fn handle_gp(_: &mut InterruptStackFrame, ec: u64) {
    use crate::{printkln, idle_forever};
    printkln!("Cannot continue: protection violation, error code {}", ec,);
    idle_forever();
}

gen_interrupt_fn!(handle_cascade, InterruptType::Cascade);
gen_interrupt_fn!(handle_uart1, InterruptType::Uart1);
gen_interrupt_fn!(handle_serial1, InterruptType::Serial1);
gen_interrupt_fn!(handle_parallel, InterruptType::Parallel);
gen_interrupt_fn!(handle_floppy, InterruptType::Floppy);
gen_interrupt_fn!(handle_lpt1, InterruptType::Lpt1);
gen_interrupt_fn!(handle_acpi, InterruptType::Acpi);
gen_interrupt_fn!(handle_open1, InterruptType::Open1);
gen_interrupt_fn!(handle_open2, InterruptType::Open2);
gen_interrupt_fn!(handle_mouse, InterruptType::Mouse);
gen_interrupt_fn!(handle_coprocessor, InterruptType::Coprocessor);
gen_interrupt_fn!(handle_primary_ata, InterruptType::PrimaryAta);
gen_interrupt_fn!(handle_secondary_ata, InterruptType::SecondaryAta);
gen_interrupt_fn!(handle_irq48, InterruptType::Irq48);
gen_interrupt_fn!(handle_irq49, InterruptType::Irq49);
gen_interrupt_fn!(handle_irq50, InterruptType::Irq50);
gen_interrupt_fn!(handle_irq51, InterruptType::Irq51);
gen_interrupt_fn!(handle_irq52, InterruptType::Irq52);
gen_interrupt_fn!(handle_irq53, InterruptType::Irq53);
gen_interrupt_fn!(handle_irq54, InterruptType::Irq54);
gen_interrupt_fn!(handle_irq55, InterruptType::Irq55);
gen_interrupt_fn!(handle_irq56, InterruptType::Irq56);
gen_interrupt_fn!(handle_irq57, InterruptType::Irq57);
gen_interrupt_fn!(handle_irq58, InterruptType::Irq58);
gen_interrupt_fn!(handle_irq59, InterruptType::Irq59);
gen_interrupt_fn!(handle_irq60, InterruptType::Irq60);
gen_interrupt_fn!(handle_irq61, InterruptType::Irq61);
gen_interrupt_fn!(handle_irq62, InterruptType::Irq62);
gen_interrupt_fn!(handle_irq63, InterruptType::Irq63);
gen_interrupt_fn!(handle_irq64, InterruptType::Irq64);
gen_interrupt_fn!(handle_irq65, InterruptType::Irq65);
gen_interrupt_fn!(handle_irq66, InterruptType::Irq66);
gen_interrupt_fn!(handle_irq67, InterruptType::Irq67);
gen_interrupt_fn!(handle_irq68, InterruptType::Irq68);
gen_interrupt_fn!(handle_irq69, InterruptType::Irq69);
gen_interrupt_fn!(handle_irq70, InterruptType::Irq70);
gen_interrupt_fn!(handle_irq71, InterruptType::Irq71);
gen_interrupt_fn!(handle_irq72, InterruptType::Irq72);
gen_interrupt_fn!(handle_irq73, InterruptType::Irq73);
gen_interrupt_fn!(handle_irq74, InterruptType::Irq74);
gen_interrupt_fn!(handle_irq75, InterruptType::Irq75);
gen_interrupt_fn!(handle_irq76, InterruptType::Irq76);
gen_interrupt_fn!(handle_irq77, InterruptType::Irq77);
gen_interrupt_fn!(handle_irq78, InterruptType::Irq78);
gen_interrupt_fn!(handle_irq79, InterruptType::Irq79);
gen_interrupt_fn!(handle_irq80, InterruptType::Irq80);
gen_interrupt_fn!(handle_irq81, InterruptType::Irq81);
gen_interrupt_fn!(handle_irq82, InterruptType::Irq82);
gen_interrupt_fn!(handle_irq83, InterruptType::Irq83);
gen_interrupt_fn!(handle_irq84, InterruptType::Irq84);
gen_interrupt_fn!(handle_irq85, InterruptType::Irq85);
gen_interrupt_fn!(handle_irq86, InterruptType::Irq86);
gen_interrupt_fn!(handle_irq87, InterruptType::Irq87);
gen_interrupt_fn!(handle_irq88, InterruptType::Irq88);
gen_interrupt_fn!(handle_irq89, InterruptType::Irq89);
gen_interrupt_fn!(handle_irq90, InterruptType::Irq90);
gen_interrupt_fn!(handle_irq91, InterruptType::Irq91);
gen_interrupt_fn!(handle_irq92, InterruptType::Irq92);
gen_interrupt_fn!(handle_irq93, InterruptType::Irq93);
gen_interrupt_fn!(handle_irq94, InterruptType::Irq94);
gen_interrupt_fn!(handle_irq95, InterruptType::Irq95);
gen_interrupt_fn!(handle_irq96, InterruptType::Irq96);
gen_interrupt_fn!(handle_irq97, InterruptType::Irq97);
gen_interrupt_fn!(handle_irq98, InterruptType::Irq98);
gen_interrupt_fn!(handle_irq99, InterruptType::Irq99);
gen_interrupt_fn!(handle_irq100, InterruptType::Irq100);
gen_interrupt_fn!(handle_irq101, InterruptType::Irq101);
gen_interrupt_fn!(handle_irq102, InterruptType::Irq102);
gen_interrupt_fn!(handle_irq103, InterruptType::Irq103);
gen_interrupt_fn!(handle_irq104, InterruptType::Irq104);
gen_interrupt_fn!(handle_irq105, InterruptType::Irq105);
gen_interrupt_fn!(handle_irq106, InterruptType::Irq106);
gen_interrupt_fn!(handle_irq107, InterruptType::Irq107);
gen_interrupt_fn!(handle_irq108, InterruptType::Irq108);
gen_interrupt_fn!(handle_irq109, InterruptType::Irq109);
gen_interrupt_fn!(handle_irq110, InterruptType::Irq110);
gen_interrupt_fn!(handle_irq111, InterruptType::Irq111);
gen_interrupt_fn!(handle_irq112, InterruptType::Irq112);
gen_interrupt_fn!(handle_irq113, InterruptType::Irq113);
gen_interrupt_fn!(handle_irq114, InterruptType::Irq114);
gen_interrupt_fn!(handle_irq115, InterruptType::Irq115);
gen_interrupt_fn!(handle_irq116, InterruptType::Irq116);
gen_interrupt_fn!(handle_irq117, InterruptType::Irq117);
gen_interrupt_fn!(handle_irq118, InterruptType::Irq118);
gen_interrupt_fn!(handle_irq119, InterruptType::Irq119);
gen_interrupt_fn!(handle_irq120, InterruptType::Irq120);
gen_interrupt_fn!(handle_irq121, InterruptType::Irq121);
gen_interrupt_fn!(handle_irq122, InterruptType::Irq122);
gen_interrupt_fn!(handle_irq123, InterruptType::Irq123);
gen_interrupt_fn!(handle_irq124, InterruptType::Irq124);
gen_interrupt_fn!(handle_irq125, InterruptType::Irq125);
gen_interrupt_fn!(handle_irq126, InterruptType::Irq126);
gen_interrupt_fn!(handle_irq127, InterruptType::Irq127);
gen_interrupt_fn!(handle_irq128, InterruptType::Irq128);
gen_interrupt_fn!(handle_irq129, InterruptType::Irq129);
gen_interrupt_fn!(handle_irq130, InterruptType::Irq130);
gen_interrupt_fn!(handle_irq131, InterruptType::Irq131);
gen_interrupt_fn!(handle_irq132, InterruptType::Irq132);
gen_interrupt_fn!(handle_irq133, InterruptType::Irq133);
gen_interrupt_fn!(handle_irq134, InterruptType::Irq134);
gen_interrupt_fn!(handle_irq135, InterruptType::Irq135);
gen_interrupt_fn!(handle_irq136, InterruptType::Irq136);
gen_interrupt_fn!(handle_irq137, InterruptType::Irq137);
gen_interrupt_fn!(handle_irq138, InterruptType::Irq138);
gen_interrupt_fn!(handle_irq139, InterruptType::Irq139);
gen_interrupt_fn!(handle_irq140, InterruptType::Irq140);
gen_interrupt_fn!(handle_irq141, InterruptType::Irq141);
gen_interrupt_fn!(handle_irq142, InterruptType::Irq142);
gen_interrupt_fn!(handle_irq143, InterruptType::Irq143);
gen_interrupt_fn!(handle_irq144, InterruptType::Irq144);
gen_interrupt_fn!(handle_irq145, InterruptType::Irq145);
gen_interrupt_fn!(handle_irq146, InterruptType::Irq146);
gen_interrupt_fn!(handle_irq147, InterruptType::Irq147);
gen_interrupt_fn!(handle_irq148, InterruptType::Irq148);
gen_interrupt_fn!(handle_irq149, InterruptType::Irq149);
gen_interrupt_fn!(handle_irq150, InterruptType::Irq150);
gen_interrupt_fn!(handle_irq151, InterruptType::Irq151);
gen_interrupt_fn!(handle_irq152, InterruptType::Irq152);
gen_interrupt_fn!(handle_irq153, InterruptType::Irq153);
gen_interrupt_fn!(handle_irq154, InterruptType::Irq154);
gen_interrupt_fn!(handle_irq155, InterruptType::Irq155);
gen_interrupt_fn!(handle_irq156, InterruptType::Irq156);
gen_interrupt_fn!(handle_irq157, InterruptType::Irq157);
gen_interrupt_fn!(handle_irq158, InterruptType::Irq158);
gen_interrupt_fn!(handle_irq159, InterruptType::Irq159);
gen_interrupt_fn!(handle_irq160, InterruptType::Irq160);
gen_interrupt_fn!(handle_irq161, InterruptType::Irq161);
gen_interrupt_fn!(handle_irq162, InterruptType::Irq162);
gen_interrupt_fn!(handle_irq163, InterruptType::Irq163);
gen_interrupt_fn!(handle_irq164, InterruptType::Irq164);
gen_interrupt_fn!(handle_irq165, InterruptType::Irq165);
gen_interrupt_fn!(handle_irq166, InterruptType::Irq166);
gen_interrupt_fn!(handle_irq167, InterruptType::Irq167);
gen_interrupt_fn!(handle_irq168, InterruptType::Irq168);
gen_interrupt_fn!(handle_irq169, InterruptType::Irq169);
gen_interrupt_fn!(handle_irq170, InterruptType::Irq170);
gen_interrupt_fn!(handle_irq171, InterruptType::Irq171);
gen_interrupt_fn!(handle_irq172, InterruptType::Irq172);
gen_interrupt_fn!(handle_irq173, InterruptType::Irq173);
gen_interrupt_fn!(handle_irq174, InterruptType::Irq174);
gen_interrupt_fn!(handle_irq175, InterruptType::Irq175);
gen_interrupt_fn!(handle_irq176, InterruptType::Irq176);
gen_interrupt_fn!(handle_irq177, InterruptType::Irq177);
gen_interrupt_fn!(handle_irq178, InterruptType::Irq178);
gen_interrupt_fn!(handle_irq179, InterruptType::Irq179);
gen_interrupt_fn!(handle_irq180, InterruptType::Irq180);
gen_interrupt_fn!(handle_irq181, InterruptType::Irq181);
gen_interrupt_fn!(handle_irq182, InterruptType::Irq182);
gen_interrupt_fn!(handle_irq183, InterruptType::Irq183);
gen_interrupt_fn!(handle_irq184, InterruptType::Irq184);
gen_interrupt_fn!(handle_irq185, InterruptType::Irq185);
gen_interrupt_fn!(handle_irq186, InterruptType::Irq186);
gen_interrupt_fn!(handle_irq187, InterruptType::Irq187);
gen_interrupt_fn!(handle_irq188, InterruptType::Irq188);
gen_interrupt_fn!(handle_irq189, InterruptType::Irq189);
gen_interrupt_fn!(handle_irq190, InterruptType::Irq190);
gen_interrupt_fn!(handle_irq191, InterruptType::Irq191);
gen_interrupt_fn!(handle_irq192, InterruptType::Irq192);
gen_interrupt_fn!(handle_irq193, InterruptType::Irq193);
gen_interrupt_fn!(handle_irq194, InterruptType::Irq194);
gen_interrupt_fn!(handle_irq195, InterruptType::Irq195);
gen_interrupt_fn!(handle_irq196, InterruptType::Irq196);
gen_interrupt_fn!(handle_irq197, InterruptType::Irq197);
gen_interrupt_fn!(handle_irq198, InterruptType::Irq198);
gen_interrupt_fn!(handle_irq199, InterruptType::Irq199);
gen_interrupt_fn!(handle_irq200, InterruptType::Irq200);
gen_interrupt_fn!(handle_irq201, InterruptType::Irq201);
gen_interrupt_fn!(handle_irq202, InterruptType::Irq202);
gen_interrupt_fn!(handle_irq203, InterruptType::Irq203);
gen_interrupt_fn!(handle_irq204, InterruptType::Irq204);
gen_interrupt_fn!(handle_irq205, InterruptType::Irq205);
gen_interrupt_fn!(handle_irq206, InterruptType::Irq206);
gen_interrupt_fn!(handle_irq207, InterruptType::Irq207);
gen_interrupt_fn!(handle_irq208, InterruptType::Irq208);
gen_interrupt_fn!(handle_irq209, InterruptType::Irq209);
gen_interrupt_fn!(handle_irq210, InterruptType::Irq210);
gen_interrupt_fn!(handle_irq211, InterruptType::Irq211);
gen_interrupt_fn!(handle_irq212, InterruptType::Irq212);
gen_interrupt_fn!(handle_irq213, InterruptType::Irq213);
gen_interrupt_fn!(handle_irq214, InterruptType::Irq214);
gen_interrupt_fn!(handle_irq215, InterruptType::Irq215);
gen_interrupt_fn!(handle_irq216, InterruptType::Irq216);
gen_interrupt_fn!(handle_irq217, InterruptType::Irq217);
gen_interrupt_fn!(handle_irq218, InterruptType::Irq218);
gen_interrupt_fn!(handle_irq219, InterruptType::Irq219);
gen_interrupt_fn!(handle_irq220, InterruptType::Irq220);
gen_interrupt_fn!(handle_irq221, InterruptType::Irq221);
gen_interrupt_fn!(handle_irq222, InterruptType::Irq222);
gen_interrupt_fn!(handle_irq223, InterruptType::Irq223);
gen_interrupt_fn!(handle_irq224, InterruptType::Irq224);
gen_interrupt_fn!(handle_irq225, InterruptType::Irq225);
gen_interrupt_fn!(handle_irq226, InterruptType::Irq226);
gen_interrupt_fn!(handle_irq227, InterruptType::Irq227);
gen_interrupt_fn!(handle_irq228, InterruptType::Irq228);
gen_interrupt_fn!(handle_irq229, InterruptType::Irq229);
gen_interrupt_fn!(handle_irq230, InterruptType::Irq230);
gen_interrupt_fn!(handle_irq231, InterruptType::Irq231);
gen_interrupt_fn!(handle_irq232, InterruptType::Irq232);
gen_interrupt_fn!(handle_irq233, InterruptType::Irq233);
gen_interrupt_fn!(handle_irq234, InterruptType::Irq234);
gen_interrupt_fn!(handle_irq235, InterruptType::Irq235);
gen_interrupt_fn!(handle_irq236, InterruptType::Irq236);
gen_interrupt_fn!(handle_irq237, InterruptType::Irq237);
gen_interrupt_fn!(handle_irq238, InterruptType::Irq238);
gen_interrupt_fn!(handle_irq239, InterruptType::Irq239);
gen_interrupt_fn!(handle_irq240, InterruptType::Irq240);
gen_interrupt_fn!(handle_irq241, InterruptType::Irq241);
gen_interrupt_fn!(handle_irq242, InterruptType::Irq242);
gen_interrupt_fn!(handle_irq243, InterruptType::Irq243);
gen_interrupt_fn!(handle_irq244, InterruptType::Irq244);
gen_interrupt_fn!(handle_irq245, InterruptType::Irq245);
gen_interrupt_fn!(handle_irq246, InterruptType::Irq246);
gen_interrupt_fn!(handle_irq247, InterruptType::Irq247);
gen_interrupt_fn!(handle_irq248, InterruptType::Irq248);
gen_interrupt_fn!(handle_irq249, InterruptType::Irq249);
gen_interrupt_fn!(handle_irq250, InterruptType::Irq250);
gen_interrupt_fn!(handle_irq251, InterruptType::Irq251);
gen_interrupt_fn!(handle_irq252, InterruptType::Irq252);
gen_interrupt_fn!(handle_irq253, InterruptType::Irq253);
gen_interrupt_fn!(handle_irq254, InterruptType::Irq254);
gen_interrupt_fn!(handle_irq255, InterruptType::Irq255);

fn is_apic_available() -> bool {
    use bit_field::BitField;
    let apic_available_in_msr = {
        let apicbase = Msr::new(0x1B);
        unsafe { apicbase.read().get_bit(11) }
    };
    apic_available_in_msr && cpuid!(1).ecx.get_bit(9)
}

fn apic_addr() -> u64 {
    use bit_field::BitField;
    let apicbase = Msr::new(0x1B);
    unsafe { apicbase.read().get_bits(12..52) }
}

fn signal_eoi(interrupt: u8) {
    if !is_apic_available() {
        if 32 <= interrupt && interrupt < 32 + 8 {
            unsafe {
                outb(0x20, 0x20);
            }
        } else if 40 <= interrupt && interrupt < 40 + 8 {
            unsafe {
                outb(0x20, 0xA0);
            }
        } else {
            unsafe {
                outb(0x20, 0x20);
                outb(0x20, 0xA0);
            }
        }
    } else {
        let addr = (apic_addr() + 0xB0) as *mut u32;
        unsafe {
            *(addr) = 0;
        }
    }
}

/// Gets the tick count that has passed since we started counting (since the RTC was set up).
/// Can handle up to 340282366920938463463374607431768211456 ticks,
/// Which, if is in microseconds, is about 10 septillion 780 sextillion years
pub fn get_tick_count() -> u128 {
    *TICK_COUNT.read()
}

/// Sleeps for the given duration of microseconds.
pub fn sleep_for(duration: u128) {
    let mut count = get_tick_count();
    let end = count + duration;
    while count != end {
        count = get_tick_count();
        x86_64::instructions::hlt();
    }
}

pub fn register_interrupt_handler(interrupt: u8, func: fn()) {
    x86_64::instructions::interrupts::disable();
    let mut tbl = IRQ_FUNCS.write();
    let irq = 32_u8.saturating_add(interrupt);
    if let Some(funcs) = tbl.get_mut(&irq) {
        funcs.resize(funcs.len() + 1, || {}).unwrap();
        funcs.pop();
        funcs.push(func).unwrap();
    }
    x86_64::instructions::interrupts::enable();
}
