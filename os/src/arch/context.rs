use super::inner::context;

#[derive(Clone)]
#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 34],
}

impl TrapContext {
    pub fn new(entry: usize, sp: usize) -> Self {
        Self { x: context::create_ctx(entry, sp) }
    }
}

extern "C" {
    pub fn __restore(ctx_addr: usize);
}

#[derive(Clone)]
#[repr(C)]
pub struct SwitchContext {
    /*
        事实上，这个上下文中保存的寄存器都是和进入和退出函数相关的寄存器
        由于某个任务背切换的位置都是在执行 __switch 时，因此 ra 寄存器保存的是退出 __switch 后的指令地址
        这样在切换回到某个任务时，调用 ret，pc 会自动指向 ra 寄存器保存地址，也就是 __switch 的下一条指令
        因为这个操作对于某个任务来说，它只能感觉到在正常执行一个函数，只不过中间被打断过，但是它是无感的
        因此只需要保存调用函数相关的寄存器即可，非常巧妙的设计
    */

    /*
        这个上下文结构也是通用的
        ra 是函数返回地址
        sp 是帧指针
        s 为调用保存寄存器
     */
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl SwitchContext {
    pub fn new(ra: usize, sp: usize) -> Self {
        Self { ra, sp, s: [0; 12] }
    }

    pub fn bare() -> Self {
        Self { ra: 0, sp: 0, s: [0; 12] }
    }

    pub fn new_with_restore_addr(sp: usize) -> Self {
        Self::new(__restore as usize, sp)
    }

    pub fn new_with_restore_addr_and_kernel_stack_sp(
        kernel_stack_start_addr: usize
    ) -> Self {
        let sp: usize = kernel_stack_start_addr - core::mem::size_of::<TrapContext>();
        Self::new(__restore as usize, sp)
    }
}

extern "C" {
    // 由于每个 CPU 寄存器存在差异，因此每个 arch 都需要使用汇编实现该函数
    pub fn __switch(current_app_ctx: *mut SwitchContext, next_app_ctx: *mut SwitchContext);
}