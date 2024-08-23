pub mod timer;

#[cfg(feature = "riscv64_qemu")]
#[path = "riscv64_qemu/mod.rs"]
mod inner;

pub fn board_init() {
    inner::plic::board_init()
}

pub fn external_irq_handler() {
    inner::plic::external_irq_handler()
}
