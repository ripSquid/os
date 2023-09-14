



use super::{MemoryAddress, PhysicalAddress, VirtualAddress, page_table::P4_TABLE};



const PAGE_SIZE: usize = 4096;
//A Physical area of memory where usize is offset by 0x1000
pub struct MemoryFrame(usize);

//A Virtual area of memory that with maps to a frame.
pub struct MemoryPage(usize);

impl MemoryFrame {
    pub fn inside_address(address: PhysicalAddress) -> Self {
        Self(address as usize / PAGE_SIZE)
    }
    pub fn starting_address(&self) -> PhysicalAddress {
        (self.0 * PAGE_SIZE) as u64
    }
}

impl MemoryPage {
    pub fn inside_address(addr: VirtualAddress) -> Self {
        // panics if address doesn't have the right sign extension
        assert!(addr < 0x0000_8000_0000_0000 || addr >= 0xffff_8000_0000_0000);
        Self(addr as usize / PAGE_SIZE)
    }
    pub fn starting_address(&self) -> VirtualAddress {
        (self.0 * PAGE_SIZE) as u64
    }
    fn p4_index(&self) -> usize {
        (self.0 >> 27) & 0o777
    }
    fn p3_index(&self) -> usize {
        (self.0 >> 18) & 0o777
    }
    fn p2_index(&self) -> usize {
        (self.0 >> 9) & 0o777
    }
    fn p1_index(&self) -> usize {
        self.0 & 0o777
    }

    pub fn translate(self) -> Option<MemoryFrame> {
        let p3 = unsafe {&*P4_TABLE}.child_table(self.p4_index());
        p3.and_then(|p3| p3.child_table(self.p3_index()))
        .and_then(|p2| p2.child_table(self.p2_index()))
        .and_then(|p1| p1[self.p1_index()].pointed_frame())
    }
}


pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<MemoryFrame>;
    fn deallocate_frame(&mut self, frame: MemoryFrame);
}
