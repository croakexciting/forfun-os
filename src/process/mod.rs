use core::borrow::Borrow;
use core::marker::PhantomData;

use crate::{
    config::*, 
    trap::context::TrapContext,
    utils::type_extern::RefCellWrap
};

use lazy_static::*;
use alloc::vec;
use alloc::vec::Vec;

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
    pub fn get_or_create(&mut self, id: usize) -> Process {
        if id >= self.apps.len() && id < MAX_APP_NUM {
            // create new app instance
            self.apps.push(Process {
                id: self.apps.len(),
                status: ProcessStatus::READY,
            });
            self.apps[self.apps.len() - 1].clone()
        } else {
            self.apps[id].clone()
        }
    }

    pub fn create(&mut self) -> Process {
        self.get_or_create(self.apps.len())
    }

    pub fn current(&mut self) -> Process {
        self.apps[self.current].clone()
    }

    pub fn next(&mut self) -> Process {
        // When the next api be called, there must be at least one apps in vector
        self.create();
        let next = (self.current + 1) % MAX_APP_NUM;
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
    status: ProcessStatus,
}

impl Process {
    pub fn set_status(&mut self, status: ProcessStatus) {
        self.status = status;
    }
}

#[derive(Copy, Clone)]
enum ProcessStatus {
    READY,
    RUNNING,
    EXITED,
}

lazy_static! {
    static ref APP_MANAGER: RefCellWrap<AppManager> = unsafe {
        // create first app
        let first_app = Process {
            id: 0,
            status: ProcessStatus::READY,
        };

        RefCellWrap::new(
            AppManager {
                current: 0,
                apps: vec![first_app],
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
                APP_START_ADDRESS + process.id * APP_SIZE
            ));
            __restore(sp);
            unreachable!()
        },
        _ => panic!("Process status is not ready"),
    }
}

pub fn start_first_app() -> ! {
    let mut manager = APP_MANAGER.exclusive_access();
    let process = manager.get_or_create(0);
    drop(manager);
    start_app(&process)
}

pub fn start_next_app() -> ! {
    // get process status
    let mut manager = APP_MANAGER.exclusive_access();
    let current = manager.current();
    manager.set_status(current.id, ProcessStatus::EXITED);
    let next = manager.create();
    println!("current appid is {}, next appid is {}", current.id, next.id);

    drop(manager);
    start_app(&next)
}
