use core::fmt::Error;
use core::fmt::{Display, Formatter};
use x86_64::registers::model_specific::*;
use x86_64::registers::rflags;

#[derive(Clone, Copy, Debug)]
pub struct CPURegs {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub cs: u16,
    pub ds: u16,
    pub ss: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
    pub rflags: u64,
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub cr8: u64,
    pub efer: u64,
    pub fsbase: u64,
    pub gsbase: u64,
    pub kgsbase: u64,
}

impl CPURegs {
    pub fn read() -> CPURegs {
        let rax: u64;
        let rbx: u64;
        let rcx: u64;
        let rdx: u64;
        let rsi: u64;
        let rdi: u64;
        let rsp: u64;
        let rbp: u64;
        let r8: u64;
        let r9: u64;
        let r10: u64;
        let r11: u64;
        let r12: u64;
        let r13: u64;
        let r14: u64;
        let r15: u64;
        let cs: u16;
        let ds: u16;
        let ss: u16;
        let es: u16;
        let fs: u16;
        let gs: u16;
        let rf: u64;
        let cr0: u64;
        let cr2: u64;
        let cr3: u64;
        let cr4: u64;
        let cr8: u64;
        let efer: u64;
        let fsbase: u64;
        let gsbase: u64;
        let kgsbase: u64;
        unsafe {
            asm!("mov %rax, $0" : "=r" (rax));
            asm!("mov %rbx, $0" : "=r" (rbx));
            asm!("mov %rcx, $0" : "=r" (rcx));
            asm!("mov %rdx, $0" : "=r" (rdx));
            asm!("mov %rsi, $0" : "=r" (rsi));
            asm!("mov %rdi, $0" : "=r" (rdi));
            asm!("mov %rsp, $0" : "=r" (rsp));
            asm!("mov %rbp, $0" : "=r" (rbp));
            asm!("mov %r8, $0" : "=r" (r8));
            asm!("mov %r9, $0" : "=r" (r9));
            asm!("mov %r10, $0" : "=r" (r10));
            asm!("mov %r11, $0" : "=r" (r11));
            asm!("mov %r12, $0" : "=r" (r12));
            asm!("mov %r13, $0" : "=r" (r13));
            asm!("mov %r14, $0" : "=r" (r14));
            asm!("mov %r15, $0" : "=r" (r15));
            asm!("mov %cs, $0" : "=r" (cs));
            asm!("mov %ds, $0" : "=r" (ds));
            asm!("mov %ss, $0" : "=r" (ss));
            asm!("mov %es, $0" : "=r" (es));
            asm!("mov %fs, $0" : "=r" (fs));
            asm!("mov %gs, $0" : "=r" (gs));
            asm!("mov %cr0, $0" : "=r" (cr0));
            asm!("mov %cr2, $0" : "=r" (cr2));
            asm!("mov %cr3, $0" : "=r" (cr3));
            asm!("mov %cr4, $0" : "=r" (cr4));
            asm!("mov %cr8, $0" : "=r" (cr8));
        }
        efer = Efer::read_raw();
        fsbase = FsBase::read().as_u64();
        gsbase = GsBase::read().as_u64();
        kgsbase = KernelGsBase::read().as_u64();
        rf = rflags::read_raw();
        CPURegs {
            rax: rax,
            rbx: rbx,
            rcx: rcx,
            rdx: rdx,
            rsi: rsi,
            rdi: rdi,
            rsp: rsp,
            rbp: rbp,
            r8: r8,
            r9: r9,
            r10: r10,
            r11: r11,
            r12: r12,
            r13: r13,
            r14: r14,
            r15: r15,
            cs: cs,
            ds: ds,
            ss: ss,
            es: es,
            fs: fs,
            gs: gs,
            rflags: rf,
            cr0: cr0,
            cr2: cr2,
            cr3: cr3,
            cr4: cr4,
            cr8: cr8,
            efer: efer,
            fsbase: fsbase,
            gsbase: gsbase,
            kgsbase: kgsbase,
        }
    }
}

impl Display for CPURegs {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(
            formatter,
            "Registers:
RAX = {:X}\tRBX = {:X}
RCX = {:X}\tRDX = {:X}
RSI = {:X}\tRDI = {:X}
RSP = {:X}\tRBP = {:X}
R8 = {:X}\tR9 = {:X}
R10 = {:X}\tR11 = {:X}
R12 = {:X}\tR13 = {:X}
R14 = {:X}\tR15 = {:X}
RFLAGS = {:X}\tCR0 = {:X}
CR2 = {:X}\tCR3 = {:X}
CR4 = {:X}\tCR8 = {:X}
EFER = {:X}
Segments:
CS = {:X}\tDS = {:X}
SS = {:X}\tES = {:X}
FS = {:X}\tGS= {:X}
FSBASE = {:X}\tGSBASE = {:X}
KERNELGSBASE = {:X}
",
            self.rax,
            self.rbx,
            self.rcx,
            self.rdx,
            self.rsi,
            self.rdi,
            self.rsp,
            self.rbp,
            self.r8,
            self.r9,
            self.r10,
            self.r11,
            self.r12,
            self.r13,
            self.r14,
            self.r15,
            self.rflags,
            self.cr0,
            self.cr2,
            self.cr3,
            self.cr4,
            self.cr8,
            self.efer,
            self.cs,
            self.ds,
            self.ss,
            self.es,
            self.fs,
            self.gs,
            self.fsbase,
            self.gsbase,
            self.kgsbase
        )?;
        Ok(())
    }
}
