#![no_std]
#![no_main]

#[macro_use]
extern crate ffos_app;

#[no_mangle]
fn main() -> i32 {
    let mut i: usize = 0;
    loop {
        i += 1;
        ffos_app::syscall::sys_nanosleep(1_000_000_000);
        println!("sleep test index {}", i);
    }
}
