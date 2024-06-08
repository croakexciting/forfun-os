#[derive(Copy, Clone)]
#[repr(C)]
pub struct SwitchContext {
    // 事实上，这个上下文中保存的寄存器都是和进入和退出函数相关的寄存器
    // 由于某个任务背切换的位置都是在执行 __switch 时，因此 ra 寄存器保存的是退出 __switch 后的指令地址
    // 这样在切换回到某个任务时，调用 ret，pc 会自动指导 ra 寄存器保存地址，也就是 __switch 的下一条指令
    // 因为这个操作对于某个任务来说，它只能感觉到在正常执行一个函数，只不过中间被打断过，但是它是无感的
    // 因此只需要保存调用函数相关的寄存器即可，非常巧妙的设计

    ra: usize,
    sp: usize,
    s: [usize; 12]
}

impl SwitchContext {
    pub fn new(ra: usize, sp: usize) -> Self {
        Self {
            ra,
            sp,
            s: [0; 12],
        }
    }

    pub fn bare() -> Self {
        Self { ra: 0, sp: 0, s: [0; 12] }
    }

    pub fn new_with_restore_addr(sp: usize) -> Self {
        extern "C" {
            fn __restore();
        }

        Self::new(__restore as usize, sp)
    }
}