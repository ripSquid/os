use core::{arch::asm, str::from_utf8_unchecked};

use crate::display::KernelDebug;

use self::features::ProcessorFeatures;
mod features;
pub struct ProcessorIdentification {
    vendor_string: CPUVendorString,
    features: ProcessorFeatures,
}
impl<'a> KernelDebug<'a> for ProcessorIdentification {
    fn debug(
        &self,
        formatter: crate::display::KernelFormatter<'a>,
    ) -> crate::display::KernelFormatter<'a> {
        formatter
            .debug_struct("CPUID")
            .debug_field("vendor", &self.vendor_string.as_str())
            .debug_field("features", &self.features.bits())
            .finish()
    }
}

impl ProcessorIdentification {
    pub fn gather() -> Self {
        let vendor_registers = unsafe { cpuid(0) };
        let feature_flags = unsafe { cpuid(1) };
        let vendor_string = CPUVendorString::from_cpuid_response(vendor_registers).unwrap();
        let features = ProcessorFeatures::from_cpuid_response(feature_flags).unwrap();
        Self {
            vendor_string,
            features,
        }
    }
    pub fn vendor(&self) -> &str {
        self.vendor_string.as_str()
    }
}
pub struct CpuIdResponse {
    call: u32,
    _eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
}

//retrieve the CPUID of the executing processor.
unsafe fn cpuid(call: u32) -> CpuIdResponse {
    let mut _eax = call;
    let mut ebx: u32 = 0;
    let mut ecx: u32 = 0;
    let mut edx: u32 = 0;
    asm! {
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

        inout("r8") _eax,
        inout("r9") ebx,
        inout("r10") ecx,
        inout("r11") edx,
    }
    CpuIdResponse {
        _eax,
        ebx,
        ecx,
        edx,
        call,
    }
}
pub struct CPUVendorString([u8; 12]);

impl CPUVendorString {
    pub fn as_str(&self) -> &str {
        unsafe { from_utf8_unchecked(&self.0) }
    }
    fn from_cpuid_response(response: CpuIdResponse) -> Option<Self> {
        (response.call == 0).then_some({
            let CpuIdResponse { ebx, ecx, edx, .. } = response;
            let first = ebx.to_le_bytes();
            let second = edx.to_le_bytes();
            let third = ecx.to_le_bytes();
            let mut iter = first.into_iter().chain(second).chain(third);
            Self(core::array::from_fn(|_| iter.next().unwrap()))
        })
    }
}
