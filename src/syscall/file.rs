use core::arch::asm;

use riscv::register::satp;
use virtio_drivers::PAGE_SIZE;

use crate::driver::block::qemu_blk::{write_block, QemuBlk};
use crate::driver::block::BlockDevice;
use crate::process::*;
use crate::arch::riscv64::copy_usize_with_user;

/// write buf of length `len`  to a file with `fd`
/// TODO: only support stdout write, modify this after add filesystem
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    // let blk_device = QemuBlk::new();
    // let buf2: [u8; PAGE_SIZE] = [1; PAGE_SIZE];
    // write_block(0, &buf2).unwrap();
    // let mut buf3: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
    // blk_device.read_block(0, &mut buf3).unwrap();
    // println!("buf3 value is {}", buf3[0]);
    write(fd, buf as *mut u8, len)
    // write_block_fn(0)
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    read(fd, buf, len)
}

pub fn sys_create_pipe(buf: *mut usize) -> isize {
    unsafe {
        let arr = core::slice::from_raw_parts_mut(buf, 2);
        let (read_end, write_end) = create_pipe(4096);
        copy_usize_with_user(read_end, &mut arr[0]);
        copy_usize_with_user(write_end, &mut arr[1]);
        0
    }
} 