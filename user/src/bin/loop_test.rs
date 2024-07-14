#![no_std]
#![no_main]

#[macro_use]
extern crate ffos_app;

#[no_mangle]
fn main() -> i32 {
    let mut i: usize = 0;
    loop {
        i = i + 1;
        if i % 1_000_000_000 == 0 {
            println!("Hello, world! +++++ 1 ++++ number {}", i);
        }
    }
}
