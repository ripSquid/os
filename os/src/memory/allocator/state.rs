use crate::display::KernelDebug;

/// The state for a given region of memory
#[derive(Default)]
pub struct PageState {
    // The size of the region this state belongs to in bytes
    size: u64,

    // The last known available offset in this state
    // where no allocation lives
    offset: u64,

    //The amount of allocations found in this state
    allocations: u64,

    _padding: u64,
}
impl<'a> KernelDebug<'a> for PageState {
    fn debug(
        &self,
        formatter: crate::display::KernelFormatter<'a>,
    ) -> crate::display::KernelFormatter<'a> {
        formatter
            .debug_struct("MemState")
            .debug_field("size", &self.size)
            .debug_field("offset", &self.offset)
            .debug_field("allocations", &self.allocations)
            .finish()
    }
}
impl PageState {
    pub fn set_size(&mut self, size: usize) {
        self.size = size as u64;
    }
    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn offset(&self) -> u64 {
        self.offset
    }
    //Mark a new allocation inside this state
    //offset is the first non inclusive address of the allocation in relation to this state
    pub fn allocate_once(&mut self, offset: u64) {
        self.allocations += 1;
        self.offset = (offset).min(self.size)
    }
    //Remove an allocation from this state
    pub fn deallocate_once(&mut self) {
        assert!(self.allocations > 0, "DEALLOCATION OVERFLOW");
        self.allocations -= 1;
        //Where have no idea where the allocation was,
        //but we know that if we don't have *any* allocations
        //this state is empty.
        if self.allocations == 0 {
            self.offset = 0;
        }
    }
    //Mark the whole state as being taken up by one big allocation
    pub fn allocate_whole(&mut self) {
        assert_eq!(self.allocations, 0, "ERROR ON ALLOCATION WHOLE");

        self.offset = self.size;
        self.allocations += 1;
    }
    //Mark the whole state as being deallocated
    pub fn deallocate_whole(&mut self) {
        assert_eq!(self.allocations, 1, "ERROR ON DEALLOCATION WHOLE");
        self.offset = 0;
        self.allocations = 0;
    }
}
