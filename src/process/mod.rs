pub mod app;
pub mod switch;
pub mod context;

use app::*;

use crate::{
    config::*, trap::context::TrapContext
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
    static ref APP_MANAGER: AppManager = unsafe {
        // create first app
        let manager = AppManager::new();
        manager
    };
}

// Default create the first app, other app created by manual
pub fn create_app(base_addr: usize) -> i32 {
    APP_MANAGER.create_app(base_addr)
}

pub fn run_apps() -> ! {
    APP_MANAGER.run_apps()
}

pub fn exit(exit_code: i32) -> ! {
    APP_MANAGER.exit(exit_code)
}

// nano time
pub fn sleep(duration: usize) {
    APP_MANAGER.sleep(duration)
}