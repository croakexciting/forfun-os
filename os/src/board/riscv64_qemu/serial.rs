use crate::{driver, process};

const UART0_ADDR: usize = 0x1000_0000;

pub fn init() -> driver::serial::Uart {
    let va = process::map_peripheral(UART0_ADDR, 4096);
    driver::serial::Uart::new(va as usize)
}