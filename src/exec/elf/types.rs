// Types
pub type elf_half = u16;
pub type elf_word = u32;
pub type elf_sword = u32;
pub type elf_xword = u64;
pub type elf_sxword = u64;
pub type elf32_addr = u32;
pub type elf32_off = u32;
pub type elf64_addr = u64;
pub type elf64_off = u64;
pub type elf32_half = elf_half;
pub type elf64_half = elf_half;
pub type elf32_word = elf_word;
pub type elf64_word = elf_word;
pub type elf32_sword = elf_sword;
pub type elf64_sword = elf_sword;

// File types
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum fileType {
None,
Rel,
Exec,
Dyn,
Core,
LoOs = 0xFE00,
HiOs = 0xFEFF,
LoProc = 0xFF00,
HiProc = 0xFFFF,
}

#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum MachineType {
None = 0,
M32 = 1,
Sparc = 2,
Intel386 = 3,
M68k = 4,
M88k = 5,
Intel486 = 6,
Intel860 = 7,
Mips = 8,
S370 = 9,
MipsRs3Le = 10,
Res011 = 11,
Res012 = 12,
Res013 = 13,
Res014 = 14,
Parisc = 15,
Res016 = 16,
Vpp550 = 17,
Sparc32plus = 18,
Intel960 = 19,
Ppc = 20,
Ppc64 = 21,
S390 = 22,
Spu = 23,
Rres024 = 24,
Rres025 = 25,
Rres026 = 26,
Rres027 = 27,
Rres028 = 28,
Rres029 = 29,
Rres030 = 30,
Rres031 = 31,
Rres032 = 32,
Rres033 = 33,
Rres034 = 34,
Rres035 = 35,
V800 = 36,
Fr20 = 37,
Rh32 = 38,
McoreRce = 39,
Arm = 40,
OldAlpha = 41,
Sh = 42,
SparcV9 = 43,
Tricore = 44,
Arc = 45,
H8300 = 46,
H8300h = 47,
H8s = 48,
H8500 = 49,
Ia64 = 50,
Mipsx = 51,
Coldfire = 52,
M68hc12 = 53,
Mma = 54,
Pcp = 55,
Ncpu = 56,
Ndr1 = 57,
Starcore = 58,
Me16 = 59,
St100 = 60,
Tinyj = 61,
X8664 = 62,
Pdsp = 63,
Pdp10 = 64,
Pdp11 = 65,
Fx66 = 66,
St9plus = 67,
St7 = 68,
M68hc16 = 69,
M68hc11 = 70,
M68hc08 = 71,
M68hc05 = 72,
Svx = 73,
St19 = 74,
Vax = 75,
Cris = 76,
Javelin = 77,
Firepath = 78,
Zsp = 79,
Mmix = 80,
Huany = 81,
Prism = 82,
Avr = 83,
Fr30 = 84,
D10v = 85,
D30v = 86,
V850 = 87,
M32r = 88,
Mn10300 = 89,
Mn10200 = 90,
Pj = 91,
OpenRisc = 92,
ArcA5 = 93,
Xtensa = 94,
Videocore = 95,
TmmGpp = 96,
Ns32k = 97,
Tpc = 98,
Snp1k = 99,
St200 = 100,
Ip2k = 101,
Max = 102,
Cr = 103,
F2mc16 = 104,
Msp430 = 105,
Blackfin = 106,
SeC33 = 107,
Sep = 108,
Arca = 109,
Unicore = 110,
Excess = 111,
Dxp = 112,
AlteraNios22 = 113,
Crx = 114,
Xgate = 115,
C166 = 116,
M16c = 117,
Dspic30f = 118,
Ce = 119,
M32c = 120,
Res121 = 121,
Res122 = 122,
Res123 = 123,
Res124 = 124,
Res125 = 125,
Res126 = 126,
Res127 = 127,
Res128 = 128,
Res129 = 129,
Res130 = 130,
Tsk3000 = 131,
Rs08 = 132,
Res133 = 133,
Ecog2 = 134,
Score = 135,
Score7 = 135,
Dsp24 = 136,
Videocore3 = 137,
Latticemico32 = 138,
SeC17 = 139,
TiC6000 = 140,
TiC2000 = 141,
TiC5500 = 142,
Res143 = 143,
Res144 = 144,
Res145 = 145,
Res146 = 146,
Res147 = 147,
Res148 = 148,
Res149 = 149,
Res150 = 150,
Res151 = 151,
Res152 = 152,
Res153 = 153,
Res154 = 154,
Res155 = 155,
Res156 = 156,
Res157 = 157,
Res158 = 158,
Res159 = 159,
MmdspPlus = 160,
CypressM8c = 161,
R32c = 162,
Trimedia = 163,
Qdsp6 = 164,
Intel8051 = 165,
Stxp7x = 166,
Nds32 = 167,
Ecog1 = 168,
Ecog1x = 168,
Maxq30 = 169,
Ximo16 = 170,
Manik = 171,
Craynv2 = 172,
Rx = 173,
Metag = 174,
McstElbrus = 175,
Ecog16 = 176,
Cr16 = 177,
Etpu = 178,
Sle9x = 179,
L1om = 180,
Intel181 = 181,
Intel182 = 182,
Res183 = 183,
Res184 = 184,
Avr32 = 185,
Stm8 = 186,
Tile64 = 187,
Tilepro = 188,
Microblaze = 189,
Cuda = 190,
Tilegx = 191,
Cloudshield = 192,
Corea1st = 193,
Corea2nd = 194,
ArcCompact2 = 195,
Open8 = 196,
Rl78 = 197,
Videocore5 = 198,
Renesas78Kor = 199,
Freescale56800ex = 200,
Ba1 = 201,
Ba2 = 202,
Xcore = 203,
MchpPic = 204,
Intel205 = 205,
Intel206 = 206,
Intel207 = 207,
Intel208 = 208,
Intel209 = 209,
Km32 = 210,
Kmx32 = 211,
Kmx16 = 212,
Kmx8 = 213,
Kvarc = 214,
Cdp = 215,
Coge = 216,
Cool = 217,
NOrc = 218,
CsrKalimba = 219,
Z80 = 220,
Visium = 221,
Ft32 = 222,
Moxie = 223,
Amdgpu = 224,
Riscv = 243,
Lanai = 244,
Ceva = 245,
CevaX2 = 246,
bpf = 247,
}

#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum Version {
None,
Current,
}

#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum IdentificationIndex {
Mag0,
Mag1,
Mag2,
Mag3,
Class,
Data,
Version,
OsAbi,
AbiVersion,
Pad,
Nident = 16,
}

static ELF_MAG0: u8 = 0x7F;
static ELF_MAG1: char = 'E';
static ELF_MAG2: char = 'L';
static ELF_MAG3: char = 'F';

#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum Class {
None,
Class32,
Class64,
}

#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum Encoding {
None,
Lsb,
Msb,
}

#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum Extension {
None,
Hpux,
Netbsd,
Linux,
Solaris = 6,
Aix,
Irix,
Freebsd,
Tru64,
Modesto,
Openbsd,
Openvms,
Nsk,
Aros,
FenixOs,
AmdGpuHsa = 64,
AmdGpuPal,
AmdGpuMesa3d,
}

const EF_AMDGPU_MACH: u8 = 0x0FF;
const EF_AMDGPU_XNACK: u16 = 0x100;
const EF_AMDGPU_MACH_NONE: u16 = 0x000;
const EF_AMDGPU_MACH_R600_R600: u16 = 0x001;
const EF_AMDGPU_MACH_R600_R630: u16 = 0x002;
const EF_AMDGPU_MACH_R600_RS880: u16 = 0x003;
const EF_AMDGPU_MACH_R600_RV670: u16 = 0x004;
const EF_AMDGPU_MACH_R600_RV710: u16 = 0x005;
const EF_AMDGPU_MACH_R600_RV730: u16 = 0x006;
const EF_AMDGPU_MACH_R600_RV770: u16 = 0x007;
const EF_AMDGPU_MACH_R600_CEDAR: u16 = 0x008;
const EF_AMDGPU_MACH_R600_CYPRESS: u16 = 0x009;
const EF_AMDGPU_MACH_R600_JUNIPER: u16 = 0x00A;
const EF_AMDGPU_MACH_R600_REDWOOD: u16 = 0x00B;
const EF_AMDGPU_MACH_R600_SUMO: u16 =  0x00C;
const EF_AMDGPU_MACH_R600_BARTS: u16 = 0x00D;
const EF_AMDGPU_MACH_R600_CAICOS: u16 = 0x00E;
const EF_AMDGPU_MACH_R600_CAYMAN: u16 = 0x00F;
const EF_AMDGPU_MACH_R600_TURKS: u16 = 0x010;
const EF_AMDGPU_MACH_R600_RESERVED_FIRST: u16 = 0x011;
const EF_AMDGPU_MACH_R600_RESERVED_LAST: u16 = 0x01f;
const EF_AMDGPU_MACH_R600_FIRST: u16 = EF_AMDGPU_MACH_R600_R600;
const EF_AMDGPU_MACH_R600_LAST: u16 =  EF_AMDGPU_MACH_R600_TURKS;
const EF_AMDGPU_MACH_AMDGCN_GFX600: u16 = 0x020;
const EF_AMDGPU_MACH_AMDGCN_GFX601: u16 = 0x021;
const EF_AMDGPU_MACH_AMDGCN_GFX700: u16 = 0x022;
const EF_AMDGPU_MACH_AMDGCN_GFX701: u16 = 0x023;
const EF_AMDGPU_MACH_AMDGCN_GFX702: u16 = 0x024;
const EF_AMDGPU_MACH_AMDGCN_GFX703: u16 = 0x025;
const EF_AMDGPU_MACH_AMDGCN_GFX704: u16 = 0x026;
const EF_AMDGPU_MACH_AMDGCN_GFX801: u16 = 0x028;
const EF_AMDGPU_MACH_AMDGCN_GFX802: u16 = 0x029;
const EF_AMDGPU_MACH_AMDGCN_GFX803: u16 = 0x02A;
const EF_AMDGPU_MACH_AMDGCN_GFX810: u16 = 0x02B;
const EF_AMDGPU_MACH_AMDGCN_GFX900: u16 = 0x02C;
const EF_AMDGPU_MACH_AMDGCN_GFX902: u16 = 0x02D;
const EF_AMDGPU_MACH_AMDGCN_GFX904: u16 = 0x02E;
const EF_AMDGPU_MACH_AMDGCN_GFX906: u16 = 0x02F;
const EF_AMDGPU_MACH_AMDGCN_RESERVED0: u16 = 0x027;
const EF_AMDGPU_MACH_AMDGCN_RESERVED1: u16 = 0x030;
const EF_AMDGPU_MACH_AMDGCN_FIRST: u16 = EF_AMDGPU_MACH_AMDGCN_GFX600;
const EF_AMDGPU_MACH_AMDGCN_LAST: u16 = EF_AMDGPU_MACH_AMDGCN_GFX906;
// Section indexes
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum SectionIndex {
Undef,
LoReserveProc = 0xFF00, // Shares LoReserve and LoProc
HiProc = 0xFF1F,
LoOs = 0xFF20,
HiOs = 0xFF3F,
Abs = 0xFFF1,
Common = 0xFFF2,
SHIndexHiReserve = 0xFFFF, // Shares SHINDEX and HIRESERVE
}

// Section types
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum SectionType {
Null,
Progbits,
Symtab,
Strtab,
Rela,
Hash,
Dynamic,
Note,
Nobits,
Rel,
Shlib,
Dynsym,
InitArray = 14,
FiniArray,
PreinitArray,
Group,
SymtabShndx,
LoOs = 0x60000000,
HiOs = 0x6FFFFFFF,
LoProc = 0x70000000,
HiProc = 0x7FFFFFFF,
LoUser = 0x80000000,
HiUser = 0xFFFFFFFF,
}

// Section attribute flags
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum SectionAttributes {
Write,
Alloc,
ExecInstr = 0x4,
Merge = 0x10,
Strings = 0x20,
InfoLink = 0x40,
LinkOrder = 0x80,
OsNonconforming = 0x100,
Group = 0x200,
Tls = 0x400,
MaskOs = 0x0FF00000,
MaskProc = 0xF0000000,
}

// Section group flags
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum GroupFlags {
Comdat = 1,
MaskOs = 0x0FF00000,
MaskProc = 0xF0000000,
}

// Symbol binding
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum SymbolBinding {
Local,
Global,
Weak,
LoOs = 10,
HiOs = 12,
MultiDefLoProc = 13, // Shares MultiDef and LoProc
HiProc = 15,
}

// Note types
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum NoteType {
AmdGpuMetadata = 1,
AmdGpuHsaMetadata = 10,
AmdGpuIsa,
AmdGpuPalMetadata,
}

#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum SymbolType {
NoType,
Object,
Func,
Section,
File,
Common,
Tls,
LoOsAmdGpuHsaKernel = 10, // Shares LoOs and AmdGpuHsaKernel
HiOs = 12,
LoProc = 13,
HiProc = 15,
}

// Symbol visibility
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum SymbolVisibility {
Default,
Internal,
Hidden,
Protected,
}

// Undefined name
static STN_UNDEF: u8 = 0;
// Relocation types
pub const R_386_NONE: u32 = 0;
pub const R_X86_64_NONE: u32 = 0;
pub const R_AMDGPU_NONE: u32 = 0;
pub const R_386_32: u32 = 1;
pub const R_X86_64_64: u32 = 1;
pub const R_AMDGPU_ABS32_LO: u32 = 1;
pub const R_386_PC32: u32 = 2;
pub const R_X86_64_PC32: u32 = 2;
pub const R_AMDGPU_ABS32_HI: u32 = 2;
pub const R_386_GOT32: u32 = 3;
pub const R_X86_64_GOT32: u32 = 3;
pub const R_AMDGPU_ABS64: u32 = 3;
pub const R_386_PLT32: u32 = 4;
pub const R_X86_64_PLT32: u32 = 4;
pub const R_AMDGPU_REL32: u32 = 4;
pub const R_386_COPY: u32 = 5;
pub const R_X86_64_COPY: u32 = 5;
pub const R_AMDGPU_REL64: u32 = 5;
pub const R_386_GLOB_DAT: u32 = 6;
pub const R_X86_64_GLOB_DAT: u32 = 6;
pub const R_AMDGPU_ABS32: u32 = 6;
pub const R_386_JMP_SLOT: u32 = 7;
pub const R_X86_64_JUMP_SLOT: u32 = 7;
pub const R_AMDGPU_GOTPCREL: u32 = 7;
pub const R_386_RELATIVE: u32 = 8;
pub const R_X86_64_RELATIVE: u32 = 8;
pub const R_AMDGPU_GOTPCREL32_LO: u32 = 8;
pub const R_386_GOTOFF: u32 = 9;
pub const R_X86_64_GOTPCREL: u32 = 9;
pub const R_AMDGPU_GOTPCREL32_HI: u32 = 9;
pub const R_386_GOTPC: u32 = 10;
pub const R_X86_64_32: u32 = 10;
pub const R_AMDGPU_REL32_LO: u32 = 10;
pub const R_386_32PLT: u32 = 11;
pub const R_X86_64_32S: u32 = 11;
pub const R_AMDGPU_REL32_HI: u32 = 11;
pub const R_X86_64_16: u32 = 12;
pub const R_X86_64_PC16: u32 = 13;
pub const R_AMDGPU_RELATIVE64: u32 = 13;
pub const R_386_TLS_TPOFF: u32 = 14;
pub const R_X86_64_8: u32 = 14;
pub const R_386_TLS_IE: u32 = 15;
pub const R_X86_64_PC8: u32 = 15;
pub const R_386_TLS_GOTIE: u32 = 16;
pub const R_X86_64_DTPMOD64: u32 = 16;
pub const R_386_TLS_LE: u32 = 17;
pub const R_X86_64_DTPOFF64: u32 = 17;
pub const R_386_TLS_GD: u32 = 18;
pub const R_X86_64_TPOFF64: u32 = 18;
pub const R_386_TLS_LDM: u32 = 19;
pub const R_X86_64_TLSGD: u32 = 19;
pub const R_386_16: u32 = 20;
pub const R_X86_64_TLSLD: u32 = 20;
pub const R_386_PC16: u32 = 21;
pub const R_X86_64_DTPOFF32: u32 = 21;
pub const R_386_8: u32 = 22;
pub const R_X86_64_GOTTPOFF: u32 = 22;
pub const R_386_PC8: u32 = 23;
pub const R_X86_64_TPOFF32: u32 = 23;
pub const R_386_TLS_GD_32: u32 = 24;
pub const R_X86_64_PC64: u32 = 24;
pub const R_386_TLS_GD_PUSH: u32 = 25;
pub const R_X86_64_GOTOFF64: u32 = 25;
pub const R_386_TLS_GD_CALL: u32 = 26;
pub const R_X86_64_GOTPC32: u32 = 26;
pub const R_386_TLS_GD_POP: u32 = 27;
pub const R_X86_64_GOT64: u32 = 27;
pub const R_386_TLS_LDM_32: u32 = 28;
pub const R_X86_64_GOTPCREL64: u32 = 28;
pub const R_386_TLS_LDM_PUSH: u32 = 29;
pub const R_X86_64_GOTPC64: u32 = 29;
pub const R_386_TLS_LDM_CALL: u32 = 30;
pub const R_X86_64_GOTPLT64: u32 = 30;
pub const R_386_TLS_LDM_POP: u32 = 31;
pub const R_X86_64_PLTOFF64: u32 = 31;
pub const R_386_TLS_LDO_32: u32 = 32;
pub const R_386_TLS_IE_32: u32 = 33;
pub const R_386_TLS_LE_32: u32 = 34;
pub const R_X86_64_GOTPC32_TLSDESC: u32 = 34;
pub const R_386_TLS_DTPMOD32: u32 = 35;
pub const R_X86_64_TLSDESC_CALL: u32 = 35;
pub const R_386_TLS_DTPOFF32: u32 = 36;
pub const R_X86_64_TLSDESC: u32 = 36;
pub const R_386_TLS_TPOFF32: u32 = 37;
pub const R_X86_64_IRELATIVE: u32 = 37;
pub const R_386_SIZE32: u32 = 38;
pub const R_386_TLS_GOTDESC: u32 = 39;
pub const R_386_TLS_DESC_CALL: u32 = 40;
pub const R_386_TLS_DESC: u32 = 41;
pub const R_386_IRELATIVE: u32 = 42;
pub const R_386_GOT32X: u32 = 43;
pub const R_X86_64_GNU_VTINHERIT: u32 = 250;
pub const R_X86_64_GNU_VTENTRY: u32 = 251;
// Segment types
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum SegmentType {
Null,
Load,
Dynamic,
Interpretor,
Note,
SharedLib,
ProgramHeader,
Tls,
LoOs = 0x60000000,
HiOs = 0x6FFFFFFF,
LoProc = 0x70000000,
HiProc = 0x7FFFFFFF,
}

// Segment flags
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum SegmentFlags {
Execute = 1,
Write = 2,
Read = 4,
MaskOs = 0x0ff00000,
MaskProc = 0xf0000000,
}

// Dynamic Array Tags
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum DynamicArrayTag {
Null,
Needed,
PltRelSz,
PltGot,
Hash,
StrTab,
SymTab,
Rela,
RelaSz,
RelaEnt,
StrSz,
SymEnt,
Init,
Fini,
SoName,
RPath,
Symbolic,
Rel,
RelSz,
RelEnt,
PltRel,
Debug,
TextRel,
JmpRel,
BindNow,
InitArray,
FiniArray,
InitArraySz,
FiniArraySz,
RunPath,
Flags,
EncodingPreInitArray = 32, // Encoding and PreInitArray shared
PreInitAraySz,
MaxPosTags,
LoOs = 0x6000000D,
HiOs = 0x6FFFF000,
LoProc = 0x70000000,
HiProc = 0x7FFFFFFF,
}

// DT_FLAGS values
#[derive(Eq, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum DynamicArrayTagFlags {
Origin = 1,
Symbolic,
TextRel = 0x4,
BindNow = 0x8,
StaticTls = 0x10,
}

// ELF file header
#[derive(Debug, Copy, Clone)]
pub struct Elf32Header {
pub e_ident: [u8; 16],
pub e_type: elf_half,
pub e_machine: elf_half,
pub e_version: elf_word,
pub e_entry: elf32_addr,
pub e_phoff: elf32_off,
pub e_shoff: elf32_off,
pub e_flags: elf_word,
pub e_ehsize: elf_half,
pub e_phentsize: elf_half,
pub e_phnum: elf_half,
pub e_shentsize: elf_half,
pub e_shnum: elf_half,
pub e_shstrndx: elf_half,
}

#[derive(Debug, Copy, Clone)]
pub struct Elf64Header {
pub e_ident: [u8; 16],
pub e_type: elf_half,
pub e_machine: elf_half,
pub e_version: elf_word,
pub e_entry: elf64_addr,
pub e_phoff: elf64_off,
pub e_shoff: elf64_off,
pub e_flags: elf_word,
pub e_ehsize: elf_half,
pub e_phentsize: elf_half,
pub e_phnum: elf_half,
pub e_shentsize: elf_half,
pub e_shnum: elf_half,
pub e_shstrndx: elf_half,
}

#[derive(Debug, Copy, Clone)]
pub struct Elf32SectionHeader {
pub sh_name: elf_word,
pub sh_type: elf_word,
pub sh_flags: elf_word,
pub sh_addr: elf32_addr,
pub sh_offset: elf32_off,
pub sh_size: elf_word,
pub sh_link: elf_word,
pub sh_info: elf_word,
pub sh_addralign: elf_word,
pub sh_entsize: elf_word,
}

#[derive(Debug, Copy, Clone)]
pub struct Elf64SectionHeader {
pub sh_name: elf_word,
pub sh_type: elf_word,
pub sh_flags: elf_word,
pub sh_addr: elf64_addr,
pub sh_offset: elf64_off,
pub sh_size: elf_word,
pub sh_link: elf_word,
pub sh_info: elf_word,
pub sh_addralign: elf_word,
pub sh_entsize: elf_word,
}

#[derive(Debug, Copy, Clone)]
pub struct Elf32ProgramHeader {
    pub p_type: elf_word,
    pub p_offset: elf32_off,
    pub p_vaddr: elf32_addr,
    pub p_paddr: elf32_addr,
    pub p_filesz: elf_word,
    pub p_memsz: elf_word,
    pub p_flags: elf_word,
    pub p_align: elf_word,
}

#[derive(Debug, Copy, Clone)]
pub struct Elf32ProgramHeader {
    pub p_type: elf_word,
    pub p_offset: elf64_off,
    pub p_vaddr: elf64_addr,
    pub p_paddr: elf64_addr,
    pub p_filesz: elf_word,
    pub p_memsz: elf_word,
    pub p_flags: elf_word,
    pub p_align: elf_word,
}

