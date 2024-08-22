use crate::{ipc::signal::{SignalFlags, SIGILL, SIGSEGV}, println, process::{app::SignalCode, back_to_idle, cow, exit, save_trap_ctx, set_signal, signal_handler}, utils::timer::set_trigger};

use super::context::TrapContext;

pub fn init() {
    /*
        初始化中，注册中断处理函数
        通常是进入一段汇编代码，在汇编中调用 irq_handler
        因此不用显式的注册 irq_handler 
     */
    super::inner::irq::init();
}

#[no_mangle]
pub fn irq_handler(ctx: &mut TrapContext) -> &mut TrapContext {
    match super::inner::irq::irq_casue() {
        IrqCause::UserEnvCall => {
            super::inner::irq::usercall(ctx);
        }
        IrqCause::Timer => {
            set_trigger();
            back_to_idle();
        }
        IrqCause::LoadFault(addr)
        | IrqCause::StoreFault(addr) => {
            if let Err(e) = cow(addr) {
                println!("[kernel] copy on write failed: {}, kernel killed it.", e);
                set_signal(None, SIGSEGV);
            } else {
                back_to_idle();
            }
        }
        IrqCause::InstructionFault => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            set_signal(None, SIGILL);
        }
        IrqCause::External => {
            // TODO: call hardware interrupt at here
        }
    }

    let signal_code = signal_handler();
    match signal_code {
        SignalCode::IGNORE => {
            // do nothing
        }
        SignalCode::Action(handler) => {
            // save ctx for sigreturn
            save_trap_ctx();
            super::inner::irq::set_singal_action(ctx, handler.handler, handler.sig);
        }
        SignalCode::KILL(e) => exit(e),
    }

    ctx
}

pub enum IrqCause {
    // exception
    UserEnvCall,
    StoreFault(usize),
    LoadFault(usize),
    InstructionFault,
    
    // interrupt
    Timer,
    External,
}