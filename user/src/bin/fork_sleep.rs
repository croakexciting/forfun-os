#![no_std]
#![no_main]

use ffos_app::syscall::*;

#[macro_use]
extern crate ffos_app;

#[no_mangle]
fn main() -> i32 {
    println!("fork syscall test");
    let mut i: usize = 0;
    let pid = sys_fork();
    if pid == 0 {
        println!("child process");
        loop {
            i = i + 1;
            sys_nanosleep(1_000_000_000);
            println!("sleep1 test index {}", i);
        }
    } else if pid > 0 {
        println!("parent process");
        loop {
            i = i + 1;
            sys_nanosleep(1_000_000_000);
            println!("sleep2 test index {}", i);
        }
    } else {
        println!("fork failed");
    }
    0
}