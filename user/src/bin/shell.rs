#![no_std]
#![no_main]

#[macro_use]
extern crate ffos_app;

extern crate alloc;

use alloc::string::String;
use ffos_app::{
    console::getchar, signal::{SignalFlags, SIGINT}, 
    syscall::*
};

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

#[no_mangle]
pub fn main() -> i32 {
    let signal = SignalFlags::SIGINT;
    sys_sigprocmask(signal.bits() as usize);
    let mut line: String = String::new();
    print!(">> ");
    loop {
        if let Some(c) = getchar() {
            match c {
                LF | CR => {
                    println!("");
                    if !line.is_empty() {
                        // get line len
                        let fd = sys_open(&line.as_str());
                        if fd < 0 {
                            line.clear();
                            println!("file not found");
                            print!(">> ");
                            continue;
                        }
                        let file_size = sys_filesize(fd as usize);
                        let block_size = (file_size as usize / 4096) + 1;
                        let buf_ptr = sys_mmap(4096 * block_size, 0x3) as usize as *mut u8;
                        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, file_size as usize)};
                        let r = sys_read(fd as usize, buf);
                        if r >= 0 {
                            let pid = sys_fork();
                            if pid == 0 {
                                sys_exec(&buf[0..file_size as usize]);
                            } else {
                                loop {
                                    if let Some(c) = getchar() {
                                        if c == 0x3 {
                                            sys_sig(pid as usize, SIGINT);
                                            continue;
                                        }
                                    }

                                    if sys_wait(pid as usize) < 0 {
                                        sys_yield()
                                    } else {
                                        break;
                                    }
                                }
                                println!("Shell: Process {} exited with code", pid);
                                sys_ummap(buf_ptr as usize);
                            }
                            line.clear();
                        } else {
                            line.clear();
                        }
                    }
                    print!(">> ");
                }
                BS | DL => {
                    if !line.is_empty() {
                        print!("{}", BS as char);
                        print!(" ");
                        print!("{}", BS as char);
                        line.pop();
                    }
                }
                _ => {
                    print!("{}", c as char);
                    line.push(c as char);
                }
            }
        } else {
            sys_yield();
        }
    }
}
