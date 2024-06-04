use super::context::SwitchContext;
use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

extern "C" {
    pub fn __switch(current_app_ctx: *mut SwitchContext, next_app_ctx: *mut SwitchContext);
}