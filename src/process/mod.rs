use crate::{
    config::*, 
    trap::context::TrapContext,
    utils::type_extern::RefCellWrap
};

use lazy_static::*;
use alloc::vec;
use alloc::vec::Vec;
use log::error;

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

struct AppManager {
    current: usize,
    apps: Vec<Process>,
}

impl AppManager {
    // pub fn app(&self)
    pub fn app(&self, id: usize) -> Process {
        self.apps[id].clone()
    }

    // return app id, if create failed, return -1
    pub fn create_app(&mut self, base_addr: usize) -> i32 {
        // just add a process at the tail
        let app_id = self.apps.len();
        if app_id < MAX_APP_NUM {
            let mut process = Process::new(app_id, base_addr);
            process.set_status(ProcessStatus::READY);
            self.apps.push(process);
            app_id as i32
        } else {
            error!("The app pool now is full, can't add new app");
            return -1;
        }
    }

    pub fn current_app(&mut self) -> Process {
        self.apps[self.current].clone()
    }

    pub fn next_app(&mut self) -> Process {
        // When the next api be called, there must be at least one apps in vector
        let next = (self.current + 1) % self.apps.len();
        self.current = next;
        self.apps[next].clone()
    }

    pub fn set_status(&mut self, id: usize, status: ProcessStatus) {
        self.apps[id].set_status(status);
    }
}

#[derive(Copy, Clone)]
struct Process {
    id: usize,
    base_address: usize,
    status: ProcessStatus,
}

impl Process {
    pub fn new(id: usize, base_addr: usize) -> Self {
        Process {
            id: id,
            base_address: base_addr,
            status: ProcessStatus::UNINIT,
        }
    }

    pub fn set_status(&mut self, status: ProcessStatus) {
        self.status = status;
    }
}

#[derive(Copy, Clone)]
enum ProcessStatus {
    UNINIT,
    READY,
    RUNNING,
    EXITED,
}

lazy_static! {
    static ref APP_MANAGER: RefCellWrap<AppManager> = unsafe {
        // create first app
        let mut first_app = Process::new(0, APP_START_ADDRESS);
        first_app.set_status(ProcessStatus::READY);
        let mut apps = vec![first_app];
        apps.reserve(MAX_APP_NUM);

        RefCellWrap::new(
            AppManager {
                current: 0,
                apps,
            }
        )   
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
