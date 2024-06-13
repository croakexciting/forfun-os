#![allow(unused)]

pub mod app;
pub mod switch;
pub mod context;

use app::*;

use crate::{
    config::*, trap::context::TrapContext
};

use lazy_static::*;

lazy_static! {
    static ref APP_MANAGER: AppManager = unsafe {
        // create first app
        let manager = AppManager::new();
        manager
    };
}

// Default create the first app, other app created by manual
pub fn create_app_with_tick(tick: usize, elf: &[u8]) -> i32 {
    APP_MANAGER.create_app(tick, elf)
}

pub fn create_app(elf: &[u8]) -> i32 {
    APP_MANAGER.create_app(5, elf)
}

pub fn activate_app(id: usize) {
    APP_MANAGER.activate_app(id);
}

pub fn run_apps() -> ! {
    APP_MANAGER.run_apps()
}

pub fn exit(exit_code: Option<i32>) -> ! {
    APP_MANAGER.exit(exit_code)
}

// nano time
pub fn sleep(duration: usize) {
    APP_MANAGER.sleep(duration)
}

pub fn back_to_idle() {
    APP_MANAGER.back_to_idle();
}