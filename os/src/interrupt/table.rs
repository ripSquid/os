use core::{mem::size_of, ops::Index};

use x86_64::{
    structures::{idt::InterruptStackFrame, DescriptorTablePointer},
    VirtAddr,
};

use super::{gatedescriptor::GateDescriptor, setup::IDTDescriptor};

pub type IFunc = extern "x86-interrupt" fn(InterruptStackFrame);
pub type EFunc = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);

#[repr(C)]
#[repr(align(16))]
pub struct IDTable {
    /// Division exception
    pub div: GateDescriptor<IFunc>,
    /// debug exception
    pub debug: GateDescriptor<IFunc>,
    /// Non-maskable interrupt
    pub nmi: GateDescriptor<IFunc>,
    /// Breakpoint exception
    pub breakpoint: GateDescriptor<IFunc>,
    /// overflow exception
    pub overflow: GateDescriptor<IFunc>,
    /// out of bounds exception
    pub oob: GateDescriptor<IFunc>,
    /// invalid op-code exception
    pub inop: GateDescriptor<IFunc>,
    /// device not available exception
    pub device_na: GateDescriptor<IFunc>,
    /// doubt fault exception
    pub double_fault: GateDescriptor<EFunc>,
    /// obsolete floating point exception (UNSUPPORTED)
    _seg_overrun: GateDescriptor<()>,
    /// invalid segment selector exception
    pub invalid_tss: GateDescriptor<EFunc>,
    /// segment not present exception
    pub seg_na: GateDescriptor<EFunc>,
    /// stack segment fault exception
    pub stack_seg_fault: GateDescriptor<EFunc>,
    /// general protection fault
    pub gpf: GateDescriptor<EFunc>,
    /// page fault
    pub page_fault: GateDescriptor<EFunc>,
    /// RESERVED
    _reserved: GateDescriptor<()>,
    /// x87 floating point error (ONLY USED IN 32-BIT MODE, UNSUPPORTED)
    _x87_fp: GateDescriptor<()>,

    /// alignment check exception
    pub alignment: GateDescriptor<EFunc>,

    /// machine check exception (MODEL SPECIFIC, UNSUPPORTED)
    pub machine_check: GateDescriptor<()>,

    /// SIMD floating point exception
    pub simd_fp: GateDescriptor<IFunc>,

    /// virtualization
    pub virtualization: GateDescriptor<IFunc>,

    /// RESERVED
    __reserved: [GateDescriptor<()>; 8],

    ///vmm communication exception
    pub vmm_communication: GateDescriptor<EFunc>,

    ///unused?
    pub security: GateDescriptor<EFunc>,

    ///RESERVED
    ___reserved: GateDescriptor<()>,

    /// user interrupts, these are for us.
    pub user_interupts: [GateDescriptor<IFunc>; 256 - 32],
}

impl IDTable {
    pub const fn new() -> Self {
        Self {
            div: GateDescriptor::null(),
            debug: GateDescriptor::null(),
            nmi: GateDescriptor::null(),
            breakpoint: GateDescriptor::null(),
            overflow: GateDescriptor::null(),
            oob: GateDescriptor::null(),
            inop: GateDescriptor::null(),
            device_na: GateDescriptor::null(),
            double_fault: GateDescriptor::null(),
            _seg_overrun: GateDescriptor::null(),
            invalid_tss: GateDescriptor::null(),
            seg_na: GateDescriptor::null(),
            stack_seg_fault: GateDescriptor::null(),
            gpf: GateDescriptor::null(),
            page_fault: GateDescriptor::null(),
            _reserved: GateDescriptor::null(),
            _x87_fp: GateDescriptor::null(),
            alignment: GateDescriptor::null(),
            machine_check: GateDescriptor::null(),
            simd_fp: GateDescriptor::null(),
            virtualization: GateDescriptor::null(),
            __reserved: [GateDescriptor::null(); 8],
            vmm_communication: GateDescriptor::null(),
            security: GateDescriptor::null(),
            ___reserved: GateDescriptor::null(),
            user_interupts: [GateDescriptor::null(); 256 - 32],
        }
    }
    pub fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            limit: (size_of::<Self>() - 1) as u16,
            base: VirtAddr::new(self as *const _ as u64),
        }
    }
}
