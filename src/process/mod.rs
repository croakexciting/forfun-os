use crate::{config::*, trap::context::TrapContext};

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

// create a new app and return the kernel stack pointer
pub fn create_app(id: usize) -> usize {
    // create a kernel stack for the app
    if !(0..4).contains(&id) {
        panic!("Now only support app id from 0 to 3");
    }

    KERNEL_STACKS[id].push_context(TrapContext::new(
        APP_START_ADDRESS + id * APP_SIZE
    ))
}

pub fn run_app(id: usize) -> ! {
    extern "C" {
        fn __restore(ctx_addr: usize);
    }

    unsafe {
        __restore(create_app(id));
        unreachable!()
    }
}