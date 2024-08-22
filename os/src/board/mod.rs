pub mod timer;

#[cfg(feature = "riscv64_qemu")]
#[path = "riscv64_qemu/mod.rs"]
mod inner;