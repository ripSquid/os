use core::marker::PhantomData;
use crate::display::KernelDebug;
use super::table::{EFunc, IFunc};

//implements the "set_function" method for a interrupt function type.
macro_rules! impl_gate_set {
        ($t:ty) => {
            impl GateDescriptor<$t> {
                    pub fn set_function(&mut self, func: $t, attributes: TypeAttribute, gdt: SegmentSelector) {
                        let addr = func as u64;
                        self.set_address(addr);
                        self.type_attributes = attributes;
                        self.selector = gdt;
                    }
                }
        };
}

#[derive(PartialEq)]
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct SegmentSelector(pub u16);

#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub struct GateDescriptor<FnPointer> {
    pub offset_low: u16,
    pub selector: SegmentSelector,
    pub type_attributes: TypeAttribute,
    pub offset_mid: u16,
    pub offset_high: u32,
    _zero: u32,
    _p: PhantomData<FnPointer>,
}
impl<T> GateDescriptor<T> {
    pub const fn null() -> Self {
        Self {
            offset_low: 0,
            selector: SegmentSelector(0),
            type_attributes: TypeAttribute(0b0000_1110_0000_0000),
            offset_mid: 0,
            offset_high: 0,
            _zero: 0,
            _p: PhantomData,
        }
    }
}

impl<'a, T> KernelDebug<'a> for GateDescriptor<T> {
    fn debug(
        &self,
        formatter: crate::display::KernelFormatter<'a>,
    ) -> crate::display::KernelFormatter<'a> {
        let address = ((self.offset_low as u64) << 0)
            | ((self.offset_mid as u64) << 16)
            | ((self.offset_high as u64) << 32);
        formatter
            .debug_struct("gate")
            .debug_field("addr", &address)
            .debug_field("seg", &self.selector.0)
            .debug_field("attr", &self.type_attributes.0)
            .finish()
    }
}

#[repr(u8)]
pub enum CPUPrivilege {
    KERNEL = 0x0,
    DRIVER1 = 0x1,
    DRIVER2 = 0x2,
    APPLICATION = 0x3,
}

#[repr(u8)]
pub enum InterruptType {
    Interrupt = 0xE,
    Trap = 0xF,
}

#[derive(Clone, Copy, Default, PartialEq)]
#[repr(transparent)]
pub struct TypeAttribute(pub u16);

impl TypeAttribute {
    pub fn new(exists: bool, cpu_privilege: CPUPrivilege, interrupt_type: InterruptType) -> Self {
        TypeAttribute(
            (exists as u16) << 15 | (cpu_privilege as u16) << 13 | (interrupt_type as u16) << 8,
        )
    }
    pub fn set_existing(&mut self, exists: bool) {
        self.0 &= 0b0111_1111_1111_1111 + ((exists as u16) << 15);
    }
}

impl<T> Default for GateDescriptor<T> {
    fn default() -> Self {
        Self::null()
    }
}

impl<T> GateDescriptor<T> {
    pub fn set_address(&mut self, address: u64) -> &mut Self {
        self.offset_low = address as u16;
        self.offset_mid = (address >> 16) as u16;
        self.offset_high = (address >> 32) as u32;
        return self;
    }
}


impl_gate_set!(IFunc);
impl_gate_set!(EFunc);
