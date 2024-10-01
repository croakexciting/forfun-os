use arm_pl011::Pl011Uart;

pub const UART0_ADDR: usize = 0x900_0000;

pub fn init_serial(addr: usize) -> Pl011Uart {
    Pl011Uart::new(addr as *mut u8)
}

// blk0
pub const BLK_HEADER_ADDR: usize = 0xA00_3E00;
