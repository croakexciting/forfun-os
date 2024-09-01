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