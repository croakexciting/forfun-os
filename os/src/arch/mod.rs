pub mod memory;
pub mod context;

#[cfg(feature = "riscv64")]
#[path = "riscv64/mod.rs"]
mod inner;

#[cfg(feature = "aarch64")]
#[path = "aarch64/mod.rs"]
mod inner;

pub fn init() {
    inner::trap::init();
}
