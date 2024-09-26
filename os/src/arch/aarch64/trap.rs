use aarch64_cpu::{asm::barrier, registers::*};
use tock_registers::interfaces::ReadWriteable;
use crate::{arch::context::TrapContext, board::{peri::GIC, timer::set_trigger}, process::{back_to_idle, cow}, syscall::syscall};
use core::arch::{asm, global_asm};

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }

    VBAR_EL1.set(__alltraps as u64);
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR);
    barrier::isb(barrier::SY);
}

#[no_mangle]
pub fn current_elx_synchronous(ctx: &mut TrapContext) -> &mut TrapContext {
    let mut esr: usize;
    unsafe { asm!("mrs {0}, ESR_EL1", out(reg) esr); }
    let ec: usize = (esr >> 26) & 0x3F;
    match ec {
        0x24 => {
            // access failed
            let addr = FAR_EL1.get() as usize;
            match cow(addr) {
                Ok(_) => {
                    back_to_idle();
                }
                Err(e) => {
                    panic!("[kernel] copy on write failed: {}, kernel killed it.", e);
                }
            }
        }
        _ => {
            panic!("current elx sync unsupported ec value: {}", ec);
        }
    }
    ctx
}

#[no_mangle]
pub fn current_elx_irq(_ctx: &mut TrapContext) {
    panic!("current elx irq");
}

#[no_mangle]
pub fn current_elx_serror(_ctx: &mut TrapContext) {
    panic!("current elx serror");
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
        0x24 => {
            // access failed
            let addr = FAR_EL1.get() as usize;
            match cow(addr) {
                Ok(_) => {
                    back_to_idle();
                }
                Err(e) => {
                    panic!("[kernel] copy on write failed: {}, kernel killed it.", e);
                }
            }
        }
        _ => {
            panic!("unsupported ec value: {}", ec);
        }
    }
    ctx
}

#[no_mangle]
pub fn lower_aarch64_irq(ctx: &mut TrapContext) -> &mut TrapContext {
    let irq_num = GIC.exclusive_access().claim();
    match irq_num {
        30 => {
            set_trigger();
            GIC.exclusive_access().complete(irq_num);
            back_to_idle();
        },
        1020.. => {},
        _ => {panic!("irq {} not supported now", irq_num);},
    }
    return ctx;
}

#[no_mangle]
pub fn lower_aarch64_serror(_ctx: &mut TrapContext) {
    panic!("lower aarch64 serror");
}