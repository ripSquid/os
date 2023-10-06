use core::{ops::Deref, str::{from_utf8, from_utf8_unchecked}, arch::asm};

use crate::display::KernelDebug;

pub struct ProcessorIdentification {
    vendor_string: CPUVendorString,
    features: ProcessorFeatures,
}
impl<'a> KernelDebug<'a> for ProcessorIdentification {
    fn debug(&self, formatter: crate::display::KernelFormatter<'a>) -> crate::display::KernelFormatter<'a> {
        formatter.debug_struct("CPUID").debug_field("vendor", &self.vendor_string.as_str()).debug_field("features", &self.features.bits()).finish()
    }
}
impl<'a> KernelDebug<'a> for &str {
    fn debug(&self, formatter: crate::display::KernelFormatter<'a>) -> crate::display::KernelFormatter<'a> {
        formatter.debug_str(self)
    }
}
impl ProcessorIdentification {
    pub fn gather() -> Self {
        let vendor_registers = unsafe { cpuid(0) };
        let feature_flags = unsafe { cpuid(1) } ;
        let vendor_string = CPUVendorString::from_cpuid_response(vendor_registers);
        let features = ProcessorFeatures::from_cpuid_response(feature_flags);
        Self { vendor_string, features }
    }
}
//Values of the EAX, EBX, ECX, and EDX registers.
type CPUIDResponse = [u32; 4];

unsafe fn cpuid(mut eax: u32) -> CPUIDResponse {
    let mut b: u32 = 0;
    let mut c: u32 = 0;
    let mut d: u32 = 0;
    asm!{
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "mov rax, r8",
        "cpuid",
        "mov r8, rax",
        "mov r9, rbx",
        "mov r10, rcx",
        "mov r11, rdx",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        
        inout("r8") eax,
        inout("r9") b,
        inout("r10") c,
        inout("r11") d,
    }
    [eax, b, c, d]
}
pub struct CPUVendorString([u8; 12]);

impl CPUVendorString {
    pub fn as_str(&self) -> &str {
        unsafe { from_utf8_unchecked(&self.0) }
    }
    fn from_cpuid_response([_eax, ebx, ecx, edx]: CPUIDResponse) -> Self {
        let first = ebx.to_le_bytes();
        let second = edx.to_le_bytes();
        let third = ecx.to_le_bytes();
        let mut iter = first.into_iter().chain(second).chain(third);
        Self(core::array::from_fn(|_| iter.next().unwrap()))
    }
}
impl ProcessorFeatures {
    fn from_cpuid_response([_eax, _ebx, ecx, edx]: CPUIDResponse) -> Self {
        let bits = (ecx as u64) + ((edx as u64) << 32); 
        Self::from_bits_retain(bits)
    }
}
bitflags! {
    #[derive(Clone, Copy, PartialEq)]
    pub struct ProcessorFeatures: u64 {
        const ECX_SSE3         = 1 << 0; 
        const ECX_PCLMUL       = 1 << 1;
        const ECX_DTES64       = 1 << 2;
        const ECX_MONITOR      = 1 << 3;  
        const ECX_DS_CPL       = 1 << 4;  
        const ECX_VMX          = 1 << 5;  
        const ECX_SMX          = 1 << 6;  
        const ECX_EST          = 1 << 7;  
        const ECX_TM2          = 1 << 8;  
        const ECX_SSSE3        = 1 << 9;  
        const ECX_CID          = 1 << 10;
        const ECX_SDBG         = 1 << 11;
        const ECX_FMA          = 1 << 12;
        const ECX_CX16         = 1 << 13; 
        const ECX_XTPR         = 1 << 14; 
        const ECX_PDCM         = 1 << 15; 
        const ECX_PCID         = 1 << 17; 
        const ECX_DCA          = 1 << 18; 
        const ECX_SSE4_1       = 1 << 19; 
        const ECX_SSE4_2       = 1 << 20; 
        const ECX_X2APIC       = 1 << 21; 
        const ECX_MOVBE        = 1 << 22; 
        const ECX_POPCNT       = 1 << 23; 
        const ECX_TSC          = 1 << 24; 
        const ECX_AES          = 1 << 25; 
        const ECX_XSAVE        = 1 << 26; 
        const ECX_OSXSAVE      = 1 << 27; 
        const ECX_AVX          = 1 << 28;
        const ECX_F16C         = 1 << 29;
        const ECX_RDRAND       = 1 << 30;
        const ECX_HYPERVISOR   = 1 << 31;

        const EDX_FPU          = 1 << (00 + 32);  
        const EDX_VME          = 1 << (01 + 32);  
        const EDX_DE           = 1 << (02 + 32);  
        const EDX_PSE          = 1 << (03 + 32);  
        const EDX_TSC          = 1 << (04 + 32);  
        const EDX_MSR          = 1 << (05 + 32);  
        const EDX_PAE          = 1 << (06 + 32);  
        const EDX_MCE          = 1 << (07 + 32);  
        const EDX_CX8          = 1 << (08 + 32);  
        const EDX_APIC         = 1 << (09 + 32);  
        const EDX_SEP          = 1 << (11 + 32); 
        const EDX_MTRR         = 1 << (12 + 32); 
        const EDX_PGE          = 1 << (13 + 32); 
        const EDX_MCA          = 1 << (14 + 32); 
        const EDX_CMOV         = 1 << (15 + 32); 
        const EDX_PAT          = 1 << (16 + 32); 
        const EDX_PSE36        = 1 << (17 + 32); 
        const EDX_PSN          = 1 << (18 + 32); 
        const EDX_CLFLUSH      = 1 << (19 + 32); 
        const EDX_DS           = 1 << (21 + 32); 
        const EDX_ACPI         = 1 << (22 + 32); 
        const EDX_MMX          = 1 << (23 + 32); 
        const EDX_FXSR         = 1 << (24 + 32); 
        const EDX_SSE          = 1 << (25 + 32); 
        const EDX_SSE2         = 1 << (26 + 32); 
        const EDX_SS           = 1 << (27 + 32); 
        const EDX_HTT          = 1 << (28 + 32); 
        const EDX_TM           = 1 << (29 + 32); 
        const EDX_IA64         = 1 << (30 + 32);
        const EDX_PBE          = 1 << (31 + 32);
    }
}