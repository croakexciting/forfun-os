#![allow(unused)]

pub mod app;
pub mod pid;

use core::usize;

use alloc::{string::String, sync::Arc, vec::Vec};
use app::*;
use spin::mutex::Mutex;

use crate::{
    arch::memory::page::{VirtAddr, VirtPage}, mm::area::UserBuffer, utils::type_extern::RefCellWrap
};

use lazy_static::*;

lazy_static! {
    static ref TASK_MANAGER: Arc<TaskManager> = unsafe { Arc::new(TaskManager::new()) };
}

// Default create the first app, other app created by manual
pub fn create_proc() -> isize {
    let elf = unsafe { core::slice::from_raw_parts(0x8100_0000 as *mut u8, 4096 * 100) };
    TASK_MANAGER.create_initproc(5, elf)
}

pub fn run_tasks() -> ! {
    TASK_MANAGER.run_task()
}

pub fn fork() -> isize {
    TASK_MANAGER.fork()
}

pub fn exec(elf: &[u8]) -> isize {
    TASK_MANAGER.exec(elf)
}

pub fn exit(exit_code: isize) -> ! {
    TASK_MANAGER.exit(exit_code)
}

// nano time
pub fn sleep(duration: usize) {
    TASK_MANAGER.sleep(duration)
}

pub fn back_to_idle() {
    TASK_MANAGER.back_to_idle();
}

pub fn cow(va: usize) -> Result<(), &'static str> {
    let vpn: VirtPage = VirtAddr::from(va).into();
    TASK_MANAGER.cow(vpn)
}

pub fn wait(pid: isize) -> isize {
    TASK_MANAGER.wait(pid)
}

pub fn write(fd: usize, buf: *mut u8, len: usize) -> isize {
    TASK_MANAGER.write(fd, buf, len)
}

pub fn create_pipe(size: usize) -> (usize, usize) {
    TASK_MANAGER.create_pipe(size)
}

pub fn read(fd: usize, buf: *mut u8, len: usize) -> isize {
    TASK_MANAGER.read(fd, buf, len)
}

pub fn open(name: String) -> isize {
    TASK_MANAGER.open(name)
}

pub fn lseek(fd: usize, seek: usize) -> isize {
    TASK_MANAGER.lseek(fd, seek)
}

pub fn sigaction(signal: usize, handler: usize) -> isize {
    TASK_MANAGER.sigaction(signal, handler)
}

pub fn set_signal(pid: Option<usize>, signal: usize) -> isize {
    TASK_MANAGER.set_signal(pid, signal)
}

pub fn set_signalmask(signal: usize) -> isize {
    TASK_MANAGER.set_signalmask(signal)
}

pub fn signal_handler() -> SignalCode {
    TASK_MANAGER.signal_handler()
}

pub fn save_trap_ctx() {
    TASK_MANAGER.save_trap_ctx()
}

pub fn sigreturn() -> isize {
    TASK_MANAGER.sigreturn()
}

pub fn getpid() -> usize {
    TASK_MANAGER.getpid()
}

pub fn mmap(size: usize, permission: usize) -> isize {
    TASK_MANAGER.mmap(size, permission)
}

pub fn ummap(addr: usize) -> isize {
    TASK_MANAGER.ummap(addr)
}

pub fn mmap_with_addr(pa: usize, size: usize, permission: usize) -> isize {
    TASK_MANAGER.mmap_with_addr(pa, size, permission)
}

pub fn shm_open(name: String, size: usize, permission: usize) -> isize {
    TASK_MANAGER.create_or_open_shm(name, size, permission)
}

pub fn sem_open(name: String) -> isize {
    TASK_MANAGER.open_sem(name)
}

pub fn sem_wait(name: String) -> isize {
    TASK_MANAGER.wait_sem(name)
}

pub fn sem_raise(name: String) -> isize {
    TASK_MANAGER.raise_sem(name)
}

pub fn create_server(name: String) -> isize {
    TASK_MANAGER.create_server(name)
}

pub fn connect_server(name: String) -> isize {
    TASK_MANAGER.connect_server(name)
}

pub fn request(coid: usize, data: Arc<Vec<u8>>) -> Option<Arc<Vec<u8>>> {
    TASK_MANAGER.request(coid, data)
}

pub fn recv_request(name: String, timeout_ms: usize) -> Option<(usize, Arc<Vec<u8>>)> {
    TASK_MANAGER.recv_request(name, timeout_ms)
}

pub fn reply_request(rcvid: usize, data: Arc<Vec<u8>>) -> isize {
    TASK_MANAGER.reply_request(rcvid, data)
}
