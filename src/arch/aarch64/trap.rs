use aarch64_cpu::{asm::barrier, registers::*};
use crate::{arch::context::TrapContext, println, syscall::syscall};
use core::arch::{asm, global_asm};

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }

    VBAR_EL1.set(__alltraps as u64);
    barrier::isb(barrier::SY);
}

#[no_mangle]
pub fn current_elx_synchronous(ctx: &mut TrapContext) {
    // let mut esr: usize;
    // unsafe { asm!("mrs {0}, ESR_EL1", out(reg) esr); }
    // let ec = (esr >> 26) & 0x3F;
    // println!("ec {}", ec);
}

#[no_mangle]
pub fn current_elx_irq(ctx: &mut TrapContext) {

}

#[no_mangle]
pub fn current_elx_serror(ctx: &mut TrapContext) {

}

#[no_mangle]
pub fn lower_aarch64_synchronous(ctx: &mut TrapContext) -> &mut TrapContext {
    let mut esr: usize;
    unsafe { asm!("mrs {0}, ESR_EL1", out(reg) esr); }
    let ec: usize = (esr >> 26) & 0x3F;
    match ec {
        0x15 => {
            ctx.x[0] = syscall(ctx.x[8], [ctx.x[0], ctx.x[1], ctx.x[2]]) as usize
        }
        _ => {
            println!("unsupported ec value: {}", ec);
        }
    }
    ctx
}

#[no_mangle]
pub fn lower_aarch64_irq(ctx: &mut TrapContext) {

}

#[no_mangle]
pub fn lower_aarch64_serror(ctx: &mut TrapContext) {
    
}