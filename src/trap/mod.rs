pub mod context;

use context::TrapContext;
use riscv::register::{
    scause::{self, Exception, Trap, Interrupt}, 
    stval, sie,
    stvec::{self, TrapMode}
};

use crate::{
    process::{back_to_idle, cow, exit}, 
    syscall::syscall, 
    utils::timer::set_trigger
};

pub fn init() {
    extern "C" {
        fn __alltraps();
    }

    // TODO: set stvec with vector mod
    // riscv 的中断处罚等级需要设置 clint 外设，并使能 sie 寄存器中 STIE 位
    // 当然也可以通过 machine level 转发，目前似乎使用的是这种，rustsbi 转发
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
pub fn trap_handler(ctx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        // TODO: 如果在调用 syscall 的时候发生任务切换，会形成 trap 嵌套，这种情况控制流是如何走，需要研究下
        // 理论上切换任务回来时会走到 back_to_idle 的下一句话。然后回会出 trap handler，接着走到 restore，后面需要详细分析下
        // 经过分析确实无法回到 syscall，所以目前的设计是不允许中断嵌套的，也就是说内核不会被 S 特权级终端打断
        // 是否被打断是由 sie 位控制的。中断嵌套功能后续实现，想到的方法应该是使用中断向量表，区分 trap 和 interrupt
        Trap::Exception(Exception::UserEnvCall) => {
            ctx.sepc += 4;
            ctx.x[10] = syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_trigger();
            back_to_idle();
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] store pagefault in application, bad addr = {:#x}, bad instruction = {:#x}.", stval, ctx.sepc);
            let r = cow(stval);
            match r {
                Ok(_) => {
                    println!("[kernel] copy on write success");
                    back_to_idle();
                }
                Err(e) => {
                    println!("[kernel] copy on write failed: {}, kernel killed it.", e);
                    exit(-1001);
                }
            }
        }
        Trap::Exception(Exception::IllegalInstruction)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::InstructionMisaligned)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit(-1002);
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}",
                scause.cause(),
                stval
            )
        }
    }

    ctx
}