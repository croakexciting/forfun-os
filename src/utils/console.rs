//! SBI console driver, for text output
use core::fmt::{self, Write};

pub fn console_putchar(c: char) {
    #[cfg(feature = "riscv64")]
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c as usize);

    #[cfg(feature = "aarch64")]
    unsafe {
        core::ptr::write_volatile(0x900_0000 as *mut char, c);   
    }
}

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

/// print string macro
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::utils::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

/// println string macro
#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::utils::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
