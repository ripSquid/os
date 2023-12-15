use super::CpuIdResponse;

impl ProcessorFeatures {
    pub fn from_cpuid_response(response: CpuIdResponse) -> Option<Self> {
        (response.call == 1).then_some({
            let CpuIdResponse { ecx, edx, .. } = response;
            let bits = (ecx as u64) + ((edx as u64) << 32);
            Self::from_bits_retain(bits)
        })
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
