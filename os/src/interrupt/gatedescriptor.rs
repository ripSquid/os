#[repr(C, packed)]
#[derive(Copy, Clone, Default)]
pub struct SegmentSelector(pub u16);

#[derive(Clone, Copy, Default)]
#[repr(C, packed)]
pub struct GateDescriptor { 
    pub offset_1: u16,
    pub selector: SegmentSelector,
    pub ist: u8,
    pub type_attributes: TypeAttribute,
    pub offset_2: u16,
    pub offset_3: u32,
    pub zero: u32,
}

#[repr(u8)]
pub enum CPUPrivilege {
        KERNEL = 0x0,
        DRIVER1 = 0x10,
        DRIVER2 = 0x20,
        APPLICATION = 0x30,
}

#[repr(u8)]
pub enum InterruptType {
        Interrupt = 0xE,
        Trap = 0xF,
}

#[derive(Clone, Copy, Default)]
struct TypeAttribute(u8);

impl TypeAttribute {
        pub fn new(exists: bool, cpu_privilege: CPUPrivilege, interrupt_type: InterruptType) -> Self {
                // 0x80 is to show it exists
                // Maybe allow selecting if the current interrupt is supposed to exist or not
                TypeAttribute((if exists {0x80} else {0}) as u8 | cpu_privilege as u8 | interrupt_type as u8)
        }
}

impl GateDescriptor {
        pub fn new(address: u64, exists: bool, cpu_privilege: CPUPrivilege, interrupt_type: InterruptType, segment_selector: SegmentSelector, ist: u8) -> Self {
                let mut gate_descriptor = Self::default().set_address(address);
                gate_descriptor.type_attributes = TypeAttribute::new(exists, cpu_privilege, interrupt_type);
                gate_descriptor.selector = segment_selector;
                gate_descriptor.ist = ist;
                return gate_descriptor;
        }

        pub fn set_address(mut self, address: u64) -> Self {
                self.offset_1 = address as u16;
                self.offset_2 = (address >> 16) as u16;
                self.offset_3 = (address >> 32) as u32;
                return self;
        }
}