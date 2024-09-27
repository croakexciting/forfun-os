use ns16550a::*;

pub const UART0_ADDR: usize = 0x1000_0000;

pub fn uart_init(baseaddr: usize) -> ns16550a::Uart {
    let uart = ns16550a::Uart::new(baseaddr);
    uart.init(WordLength::EIGHT,
        StopBits::ONE,
        ParityBit::DISABLE,
        ParitySelect::EVEN,
        StickParity::DISABLE,
        Break::DISABLE,
        DMAMode::MODE0,
        Divisor::BAUD115200,
    );
    uart
}