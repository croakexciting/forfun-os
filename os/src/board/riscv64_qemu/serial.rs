use crate::{driver, process};

pub const UART0_ADDR: usize = 0x1000_0000;

pub fn init() -> driver::serial::Uart {
    driver::serial::Uart::new(UART0_ADDR as usize)
}