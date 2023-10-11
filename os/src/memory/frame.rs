use crate::display::KernelDebug;

use super::{ElfTrustAllocator, MemoryAreaIter, PhysicalAddress, PAGE_SIZE_4K};

//A Physical area of memory where usize is offset by 0x1000
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryFrame(usize);

impl MemoryFrame {
    #[inline]
    pub fn inside_address(address: PhysicalAddress) -> Self {
        Self(address as usize / PAGE_SIZE_4K)
    }
    #[inline]
    pub fn starting_address(&self) -> PhysicalAddress {
        (self.0 * PAGE_SIZE_4K) as u64
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<MemoryFrame>;
    fn deallocate_frame(&mut self, frame: MemoryFrame);
}

impl<'a> KernelDebug<'a> for MemoryFrame {
    fn debug(
        &self,
        formatter: crate::display::KernelFormatter<'a>,
    ) -> crate::display::KernelFormatter<'a> {
        formatter
            .debug_struct("frame:")
            .debug_field("", &self.0)
            .finish()
    }
}
impl<'a, T: KernelDebug<'a>> KernelDebug<'a> for Option<T> {
    fn debug(
        &self,
        formatter: crate::display::KernelFormatter<'a>,
    ) -> crate::display::KernelFormatter<'a> {
        match self {
            Some(value) => value.debug(formatter.debug_str("Some(")).debug_str(")"),
            None => formatter.debug_str("None"),
        }
    }
}

impl Iterator for FrameIter {
    type Item = MemoryFrame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame.0 <= self.end {
            self.frame.0 += 1;
            Some(MemoryFrame(self.frame.0 - 1))
        } else {
            None
        }
    }
}
pub struct FrameIter {
    frame: MemoryFrame,
    start: usize,
    end: usize,
}

pub struct FrameRangeInclusive {
    start_frame: usize,
    end_frame: usize,
}

impl FrameRangeInclusive {
    pub fn contains(&self, frame: &MemoryFrame) -> bool {
        (self.start_frame..=self.end_frame).contains(&frame.0)
    }
    pub fn new(start: MemoryFrame, end: MemoryFrame) -> Self {
        Self {
            start_frame: start.0,
            end_frame: end.0,
        }
    }
    pub fn span(&self) -> usize {
        (self.end_frame + 1).saturating_sub(self.start_frame)
    }
}
impl<'a> KernelDebug<'a> for FrameRangeInclusive {
    fn debug(
        &self,
        formatter: crate::display::KernelFormatter<'a>,
    ) -> crate::display::KernelFormatter<'a> {
        formatter
            .debug_num(self.start_frame)
            .debug_str("..=")
            .debug_num(self.end_frame)
    }
}
impl IntoIterator for FrameRangeInclusive {
    type Item = MemoryFrame;

    type IntoIter = FrameIter;

    fn into_iter(self) -> Self::IntoIter {
        FrameIter {
            frame: MemoryFrame(self.start_frame),
            start: self.start_frame,
            end: self.end_frame,
        }
    }
}

impl FrameAllocator for ElfTrustAllocator {
    fn allocate_frame(&mut self) -> Option<MemoryFrame> {
        let Some(area) = self.active_area else {
            return None;
        };

        let frame = self.next_free_frame.clone();
        let calf = {
            let addr = area.base_address + area.length - 1;
            MemoryFrame::inside_address(addr)
        };

        if frame > calf {
            self.choose_next_area();
        } else if self.kernel.contains(&frame) {
            self.next_free_frame = MemoryFrame(self.kernel.end_frame + 1);
        } else if self.multiboot.contains(&frame) {
            self.next_free_frame = MemoryFrame(self.multiboot.end_frame + 1);
        } else {
            self.next_free_frame.0 += 1;
            self.available_frames -= 1;
            return Some(frame);
        }
        self.allocate_frame()
    }

    fn deallocate_frame(&mut self, _frame: MemoryFrame) {
        unimplemented!("the ElfTrustAllocator Doesn't keep track of allocated frames!")
    }
}
impl ElfTrustAllocator {
    fn choose_next_area(&mut self) {
        self.active_area = self
            .areas
            .clone()
            .filter(|area| {
                let address = area.base_address + area.length - 1;
                MemoryFrame::inside_address(address) >= self.next_free_frame
            })
            .min_by_key(|area| area.base_address);

        if let Some(area) = self.active_area {
            let start_frame = MemoryFrame::inside_address(area.base_address);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
    pub fn new(
        kernel: FrameRangeInclusive,
        multiboot: FrameRangeInclusive,
        areas: MemoryAreaIter,
    ) -> Self {
        let available_frames = {
            let total = areas
                .clone()
                .fold(0usize, |acc, entry| (entry.length as usize / 4096) + acc);
            total - kernel.span() - multiboot.span()
        };
        let mut ourself = Self {
            next_free_frame: MemoryFrame::inside_address(0),
            active_area: None,
            areas,
            multiboot,
            kernel,
            available_frames,
        };
        ourself.choose_next_area();
        ourself
    }
    pub fn available_frames_left(&self) -> usize {
        self.available_frames
    }
}
