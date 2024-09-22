pub mod timer;

#[cfg(feature = "riscv64_qemu")]
#[path = "riscv64_qemu/mod.rs"]
pub mod peri;

#[cfg(feature = "aarch64_qemu")]
#[path = "aarch64_qemu/mod.rs"]
pub mod peri;

pub fn console_putchar(c: char) {
    peri::console_putchar(c)
}

pub fn console_getchar() -> u8 {
    peri::console_getchar()
}

#[allow(unused)]
pub fn external_irq_handler() {
    peri::plic::external_irq_handler()
}

pub fn board_init() {
    peri::enable_virtual_mode();
    // peri::board_init()
}

pub fn shutdown(failure: bool) -> ! {
    peri::shutdown(failure)
}