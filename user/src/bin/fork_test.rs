#![no_std]
#![no_main]

use ffos_app::syscall::{sys_fork, sys_wait, sys_yield};

#[macro_use]
extern crate ffos_app;

#[no_mangle]
fn main() -> i32 {
    println!("fork syscall test");
    let pid = sys_fork();
    if pid == 0 {
        println!("child process");
    } else if pid > 0 {
        println!("parent process");
        loop {
            if sys_wait(pid as usize) < 0 {
                sys_yield();
            } else {
                println!("child process done");
                break;
            }
        }
    } else {
        println!("fork failed");
    }
    0
}