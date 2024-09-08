use aarch64_cpu::{asm::barrier, registers::*};
use tock_registers::interfaces::ReadWriteable;
use crate::{arch::context::TrapContext, println, syscall::syscall};
use core::arch::{asm, global_asm};

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }

    VBAR_EL1.set(__alltraps as u64);
    // CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR);
    barrier::isb(barrier::SY);
}

#[no_mangle]
pub fn current_elx_synchronous(_ctx: &mut TrapContext) {
    println!("current elx sync");
}

#[no_mangle]
pub fn current_elx_irq(_ctx: &mut TrapContext) {
    println!("current elx irq");
}

#[no_mangle]
pub fn current_elx_serror(_ctx: &mut TrapContext) {
    println!("current elx serror");
}

#[no_mangle]
pub fn lower_aarch64_synchronous(ctx: &mut TrapContext) -> &mut TrapContext {
    let mut esr: usize;
    unsafe { asm!("mrs {0}, ESR_EL1", out(reg) esr); }
    let ec: usize = (esr >> 26) & 0x3F;
    match ec {
        0x15 => {
            ctx.x[0] = syscall(ctx.x[8], [ctx.x[0], ctx.x[1], ctx.x[2], ctx.x[3]]) as usize
        }
        _ => {
            println!("unsupported ec value: {}", ec);
        }
    }
    ctx
}

#[no_mangle]
pub fn lower_aarch64_irq(_ctx: &mut TrapContext) {
    println!("lower aarch64 irq");
}

#[no_mangle]
pub fn lower_aarch64_serror(_ctx: &mut TrapContext) {
    println!("lower aarch64 serror");
}