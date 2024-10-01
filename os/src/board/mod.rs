pub mod timer;

#[cfg(feature = "riscv64_qemu")]
#[path = "riscv64_qemu/mod.rs"]
pub mod inner;

#[cfg(feature = "aarch64_qemu")]
#[path = "aarch64_qemu/mod.rs"]
pub mod inner;

pub fn console_putchar(c: char) {
    inner::console_putchar(c)
}

pub fn console_getchar() -> u8 {
    inner::console_getchar()
}

#[allow(unused)]
pub fn external_irq_handler() {
    inner::interrupt::external_irq_handler()
}

pub fn board_init() {
    inner::board_init()
}

pub fn shutdown(failure: bool) -> ! {
    inner::shutdown(failure)
}