use core::marker::PhantomData;

use x86_64::structures::idt::InterruptStackFrame;

use crate::display::{
    macros::{print_hex, print_str},
    KernelDebug,
};

use super::table::{IFunc, EFunc};

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct SegmentSelector(pub u16);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GateDescriptor<FnPointer> {
    pub offset_low: u16,
    pub selector: SegmentSelector,
    pub ist: u8,
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
            ist: 0,
            type_attributes: TypeAttribute(0b0000_1110),
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

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct TypeAttribute(pub u8);

impl TypeAttribute {
    pub fn new(exists: bool, cpu_privilege: CPUPrivilege, interrupt_type: InterruptType) -> Self {
        TypeAttribute((exists as u8) << 7 | (cpu_privilege as u8) << 5 | interrupt_type as u8)
    }
    pub fn set_existing(&mut self, exists: bool) {
        self.0 &= 0b0111_1111 + ((exists as u8) << 7);
    }
}

impl<T> Default for GateDescriptor<T>{
        fn default() -> Self {
                Self::null()
        }
}

impl<T> GateDescriptor<T> {
    pub fn new(
        address: u64,
        exists: bool,
        cpu_privilege: CPUPrivilege,
        interrupt_type: InterruptType,
        segment_selector: SegmentSelector,
        ist: u8,
    ) -> Self {
        let mut gate_descriptor = Self::default();
        gate_descriptor.set_address(address);
        gate_descriptor.type_attributes = TypeAttribute::new(exists, cpu_privilege, interrupt_type);
        gate_descriptor.selector = segment_selector;
        gate_descriptor.ist = ist;
        return gate_descriptor;
    }

    pub fn set_address(&mut self, address: u64) -> &mut Self {
        //print_str!("supposed to be");
        //print_hex!(address);
        self.offset_low = address as u16;
        self.offset_mid = (address >> 16) as u16;
        self.offset_high = (address >> 32) as u32;
        //print_hex!((self.offset_3 as u64) << 32 | (self.offset_2 as u64) << 16 | (self.offset_1 as u64));
        return self;
    }
}

pub trait GateDescriptorFunction {
        fn as_addr(&self) -> u64;
}
impl GateDescriptorFunction for IFunc {
    fn as_addr(&self) -> u64 {
        self as *const _ as u64
    }
}
impl GateDescriptorFunction for EFunc {
        fn as_addr(&self) -> u64 {
                self as *const _ as u64
        }
}
impl<T: GateDescriptorFunction> GateDescriptor<T> {
    pub fn set_function(&mut self, func: T, attributes: TypeAttribute, gdt: SegmentSelector) {
        let addr = func.as_addr();
        self.set_address(addr);
        self.type_attributes = attributes;
        self.selector = gdt;

    }
}