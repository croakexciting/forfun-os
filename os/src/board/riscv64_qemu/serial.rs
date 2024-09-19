use crate::driver;

pub const UART0_ADDR: usize = 0x1000_0000;

pub fn init() -> driver::serial::qemu_serial::Uart {
    driver::serial::qemu_serial::Uart::new(UART0_ADDR as usize)
}