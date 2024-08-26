pub mod timer;

#[cfg(feature = "riscv64_qemu")]
#[path = "riscv64_qemu/mod.rs"]
pub mod peri;

pub fn console_putchar(c: char) {
    peri::CONSOLE.exclusive_access().put(c);
}

pub fn console_getchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}

pub fn external_irq_handler() {
    // inner::plic::external_irq_handler()
}

pub fn enable_virtual_mode() {
    peri::enable_virtual_mode()
}
