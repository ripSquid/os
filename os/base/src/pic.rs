use pic8259::ChainedPics;

pub static mut pics: ChainedPics = unsafe { ChainedPics::new(0x20, 0x28) };