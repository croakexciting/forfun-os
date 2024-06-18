#![allow(unused)]

pub mod app;
pub mod switch;
pub mod context;
pub mod pid;

use app::*;
use alloc::sync::Arc;
use spin::mutex::Mutex;

use crate::{
    mm::basic::VirtAddr, trap::context::TrapContext, utils::type_extern::RefCellWrap
};

use lazy_static::*;

lazy_static! {
    static ref TASK_MANAGER: Arc<TaskManager> = unsafe {
        Arc::new(TaskManager::new())
    };
}

// Default create the first app, other app created by manual
pub fn create_proc() -> isize {
    let elf = unsafe { core::slice::from_raw_parts(0x8100_0000 as *mut u8, 4096*100)};
    TASK_MANAGER.create_initproc(5, elf)
}

pub fn run_tasks() -> ! {
    TASK_MANAGER.run_task()
}

pub fn fork() -> isize {
    println!("fork!!!");
    TASK_MANAGER.fork();
    println!("back to fork");
    0
}

pub fn exit(exit_code: isize) -> ! {
    TASK_MANAGER.exit(exit_code)
}

// nano time
pub fn sleep(duration: usize) {
    TASK_MANAGER.sleep(duration)
}

pub fn back_to_idle() {
    println!("back to idle");
    TASK_MANAGER.back_to_idle();
}

pub fn remap(va: usize) -> Result<(), &'static str> {
    println!("remap");
    TASK_MANAGER.remap(VirtAddr::from(va).into())
}