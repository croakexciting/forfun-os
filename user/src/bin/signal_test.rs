#![no_std]
#![no_main]

use ffos_app::syscall::*;
use ffos_app::signal::*;

#[macro_use]
extern crate ffos_app;

fn handler() {
    println!("user signal handler called");
    sys_sigreturn();
}

#[no_mangle]
fn main() -> i32 {
    let pid = sys_fork();
    sys_sigaction(SIGUSR1, handler as usize);
    
    if pid == 0 {
        loop {
            println!("child process");
            sys_nanosleep(1_000_000_000)
        } 
        // sys_exec(0x8200_0000);
    } else if pid > 0 {
        println!("parent process");
        loop {
            if sys_wait(pid as usize) < 0 {
                sys_nanosleep(1_000_000_000);
                sys_kill(pid as usize, SIGUSR1);
                sys_nanosleep(1_000_000_000);
                sys_kill(pid as usize, SIGINT);                
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
