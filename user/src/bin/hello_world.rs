#![no_std]
#![no_main]

#[macro_use]
extern crate ffos_app;

#[no_mangle]
fn main() -> i32 {
    println!("Hello, world!");
    0
}
