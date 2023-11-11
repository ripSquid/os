#[derive(Default)]
pub struct PageState {
    size: u64,
    offset: u64,
    allocations: u64,
    _padding: u64,
}
impl PageState {
    pub fn _is_null(&self) -> bool {
        self.size == 0
    }
    pub fn set_size(&mut self, size: usize) {
        self.size = size as u64;
    }
    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn offset(&self) -> u64 {
        self.offset
    }
    pub fn allocate_once(&mut self, offset: u64) {
        self.allocations += 1;
        self.offset = (offset).min(self.size)
    }
    pub fn deallocate_once(&mut self) {
        self.allocations -= 1;
        if self.allocations == 0 {
            self.offset = 0;
        }
    }
    pub fn allocate_whole(&mut self) {
        assert_eq!(self.allocations, 0);
        self.offset = self.size;
        self.allocations += 1;
    }
    pub fn deallocate_whole(&mut self) {
        assert_eq!(self.allocations, 1);
        self.offset = 0;
        self.allocations = 0;
    }
}
