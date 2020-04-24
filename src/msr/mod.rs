/// Many MSRs have carried over from one generation of IA-32 processors to the next and to Intel 64 processors. A
/// subset of MSRs and associated bit fields, which do not change on future processor generations, are now considered
/// architectural MSRs. For historical reasons (beginning with the Pentium 4 processor), these "architectural MSRs"
/// were given the prefix "IA32_".
/// Code that accesses a machine specified MSR and that is executed on a processor that does not support that MSR
/// will generate an exception.
/// Architectural MSR or individual bit fields in an architectural MSR may be introduced or transitioned at the granularity
/// of certain processor family/model or the presence of certain CPUID feature flags.
/// MSR address range between 40000000H - 400000FFH is marked as a specially reserved range. All existing and
/// future processors will not implement any features using any MSR in this range.
pub mod arch;
