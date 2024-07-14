#![no_std]
#![no_main]

#[macro_use]
extern crate ffos_app;

extern crate alloc;

use alloc::string::String;
use ffos_app::{console::getchar, syscall::{sys_connect_server, sys_exec, sys_fork, sys_mmap, sys_request_server, sys_ummap, sys_wait, sys_yield}};

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

const READ_FILE_SERVER: &'static str = "read_file\0"; 

fn get_elf(name: &str, elf: &mut [u8]) -> isize {
    let coid = sys_connect_server(&READ_FILE_SERVER);
    if coid >= 0 {
        return sys_request_server(coid as usize, name.as_bytes(), elf.as_mut_ptr());
    } else {
        return coid;
    }
}

#[no_mangle]
pub fn main() -> i32 {
    let mut line: String = String::new();
    print!(">> ");
    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");
                if !line.is_empty() {
                    // 如果超过 1M，直接拉倒
                    let buf_ptr = sys_mmap(4096 * 256, 0x3) as usize as *mut u8;
                    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, 4096 * 256)};
                    let elf_len = get_elf(&line.as_str(), buf) as usize;
                    let pid = sys_fork();
                    if pid == 0 {
                        sys_exec(&buf[0..elf_len]);
                    } else {
                        loop {
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
    }
}
