use core::arch::asm;

const PIT_CHANNEL0_DATA: u16 = 0x40;
const PIT_CHANNEL1_DATA: u16 = 0x41;
const PIT_CHANNEL2_DATA: u16 = 0x42;
const PIT_COMMAND_REG: u16 = 0x43;

unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
    );
}

pub unsafe fn pitinit(freq: u32) {
    let divisor: u16 = (1193180u32 / freq).try_into().unwrap();

    let command: u8 = (1 << 4) | (1 << 5) | ((3) << 1);

    outb(PIT_COMMAND_REG, command);

    outb(PIT_CHANNEL0_DATA, divisor as u8);
    outb(PIT_CHANNEL0_DATA, (divisor >> 8) as u8);
}
