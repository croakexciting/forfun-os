#[cfg(feature = "riscv64")]
#[path = "riscv64/mod.rs"]
mod inner;

#[cfg(feature = "aarch64")]
#[path = "aarch64/mod.rs"]
mod inner;

pub mod context;

pub fn init() {
    inner::trap::init();
}

pub fn shutdown(failure: bool) -> ! {
    inner::shutdown(failure)
}

extern "C" {
    pub fn __restore(ctx_addr: usize);
}

pub const KERNEL_STACK_SIZE: usize = inner::config::KERNEL_STACK_SIZE;
pub const MAX_APP_NUM: usize = inner::config::MAX_APP_NUM;
pub const APP_START_ADDRESS: usize = inner::config::APP_START_ADDRESS;
pub const APP_SIZE: usize = inner::config::APP_SIZE;