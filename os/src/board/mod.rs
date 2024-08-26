use crate::driver::block::qemu_blk::QemuBlk;

pub mod timer;

#[cfg(feature = "riscv64_qemu")]
#[path = "riscv64_qemu/mod.rs"]
mod inner;

pub fn console_putchar(c: char) {
    inner::CONSOLE.exclusive_access().put(c);
}

pub fn console_getchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}

pub fn external_irq_handler() {
    // inner::plic::external_irq_handler()
}

pub fn enable_virtual_mode() {
    inner::enable_virtual_mode()
}

pub fn init_blk0() -> QemuBlk {
    inner::init_blk0()
}