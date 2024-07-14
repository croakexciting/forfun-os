use super::context::SwitchContext;

extern "C" {
    pub fn __switch(current_app_ctx: *mut SwitchContext, next_app_ctx: *mut SwitchContext);
}