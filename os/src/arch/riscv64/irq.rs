use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval,
    stvec::{self, TrapMode},
};

use crate::{
    arch::{context::TrapContext, irq::IrqCause}, syscall::syscall
};

pub fn init() {
    extern "C" {
        fn __alltraps();
    }

    // TODO: set stvec with vector mod
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
        sie::set_stimer();
    }
}

pub fn irq_casue() -> IrqCause {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => IrqCause::UserEnvCall,
        Trap::Interrupt(Interrupt::SupervisorTimer) => IrqCause::Timer,
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault) => IrqCause::StoreFault(stval),
        Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => IrqCause::LoadFault(stval),
        Trap::Exception(Exception::IllegalInstruction)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::InstructionMisaligned) => IrqCause::InstructionFault,
        Trap::Interrupt(Interrupt::SupervisorExternal) => IrqCause::External,
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}",
                scause.cause(),
                stval
            )
        }
    }
}

pub fn usercall(ctx: &mut TrapContext) {
    ctx.x[33] += 4;
    ctx.x[10] = syscall(
        ctx.x[17], 
        [ctx.x[10], ctx.x[11], ctx.x[12], ctx.x[13]]
    ) as usize;
}

pub fn set_singal_action(ctx: &mut TrapContext, pc: usize, sig: usize) {
    ctx.x[33] = pc;
    ctx.x[10] = sig;
}