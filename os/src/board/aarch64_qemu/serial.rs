use arm_pl011::Pl011Uart;
pub const UART0_ADDR: usize = 0x900_0000;

pub fn init(addr: usize) -> Pl011Uart {
    Pl011Uart::new(addr as *mut u8)
}