pub mod app;
pub mod switch;
pub mod context;

use core::borrow::BorrowMut;

use app::*;
use switch::__switch;

use crate::{
    config::*, process, trap::context::TrapContext, utils::type_extern::RefCellWrap
};

use lazy_static::*;

// 内核需要为每个应用提供独立的内核栈
// 这些栈既不在栈空间，也不在堆空间，而是直接从物理内存上分配一块固定大小且连续的区域, Linux 默认为 16KB，我们暂时使用 4KB
// 由于目前我们没有内存管理，所以使用静态变量放在 data 段
#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

// 简单考虑，固定进程数量
static KERNEL_STACKS: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

impl KernelStack {
    fn sp(&self) -> usize {
        // 栈是从高向低增长，所以初始 sp 是这段空间的最高地址
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    pub fn push_context(&self, ctx: TrapContext) -> usize {
        let trap_ctx_ptr = (self.sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_ctx_ptr = ctx;
        }
        trap_ctx_ptr as usize
    }
}

lazy_static! {
    static ref APP_MANAGER: RefCellWrap<AppManager> = unsafe {
        // create first app
        let mut manager = AppManager::new();
        manager.create_app(APP_START_ADDRESS);
        RefCellWrap::new(manager)
    };
}

fn start_app(process: &Process) -> ! {
    extern "C" {
        fn __restore(ctx_addr: usize);
    }

    match process.status {
        ProcessStatus::READY => unsafe {
            let sp = KERNEL_STACKS[process.id].push_context(TrapContext::new(
                process.base_address
            ));
            let mut manager = APP_MANAGER.exclusive_access();
            manager.set_status(process.id, ProcessStatus::RUNNING);
            drop(manager);
            __restore(sp);
            unreachable!()
        },
        _ => panic!("Process status is not ready"),
    }
}

// Default create the first app, other app created by manual
pub fn create_app(base_addr: usize) -> i32 {
    let mut manager = APP_MANAGER.exclusive_access();
    manager.create_app(base_addr)
}

pub fn start_first_app() -> ! {
    let manager = APP_MANAGER.exclusive_access();
    let process = manager.app(0);
    drop(manager);
    start_app(&process)
}

pub fn start_next_app() -> ! {
    // get process status
    let mut manager = APP_MANAGER.exclusive_access();
    let current = manager.current_app();
    manager.set_status(current.id, ProcessStatus::EXITED);
    let next = manager.next_app();
    println!("current appid is {}, next appid is {}", current.id, next.id);

    drop(manager);
    start_app(&next)
}

pub fn run_next() {
    let mut manager = APP_MANAGER.exclusive_access();

    // 创建一个 idle app，这个 app 的状态始终是 running，但是运行到它的时候，就直接 yield 出去
    // 使得内核中始终保持一个进程的存在
    
}

pub fn run_apps() {
    use ProcessStatus::*;
    loop {
        let mut manager = APP_MANAGER.exclusive_access();
        let process = manager.next_app().borrow_mut();
        let idle_ctx: *mut context::SwitchContext = manager.idle_ctx();
        match process.status {
            READY => unsafe {
                __switch(idle_ctx, &mut process.ctx as *mut _);
            },
            RUNNING => {},
            _ => {},
        }
    }
}
