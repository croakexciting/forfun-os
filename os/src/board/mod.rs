pub mod timer;

#[cfg(feature = "riscv64_qemu")]
#[path = "riscv64_qemu/mod.rs"]
mod inner;

use lazy_static::*;

#[cfg(feature = "riscv64_qemu")]
lazy_static! {
    pub static ref 
}

pub fn console_putchar(c: char) {
    inner::CONSOLE.exclusive_access().put(c);
}

pub fn console_getchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}
