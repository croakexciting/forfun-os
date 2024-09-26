#[cfg(feature = "riscv64_qemu")]
pub mod plic;

#[cfg(feature = "aarch64_qemu")]
pub mod gicv2;