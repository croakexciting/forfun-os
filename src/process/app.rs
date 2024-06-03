use core::cell::RefMut;

use crate::config::*;
use crate::trap::context::TrapContext;
use crate::utils::type_extern::RefCellWrap;

use alloc::vec;
use alloc::vec::Vec;
use log::error;

use super::context::SwitchContext;
use super::KERNEL_STACKS;


pub struct AppManagerInner {
    current: usize,
    apps: Vec<Process>,
    idle_ctx: SwitchContext,
}

impl AppManagerInner {
    pub fn new() -> Self {
        AppManagerInner {
            current: 0,
            apps: vec![],
            // idle process is a unstop loop process
            idle_ctx: SwitchContext::new(0, 0),
        }
    }

    // pub fn app(&self)
    pub fn app(&mut self, id: usize) -> RefMut<Process> {
        self.apps[id].exclusive_access()
    }

    // get idle ctx
    pub fn idle_ctx(&mut self) -> *mut SwitchContext {
        &mut self.idle_ctx as *mut _
    }

    // return app id, if create failed, return -1
    pub fn create_app(&mut self, base_addr: usize) -> i32 {
        // just add a process at the tail
        let app_id = self.apps.len();
        if app_id < MAX_APP_NUM {
            let mut process = Process::new(app_id, base_addr);
            process.set_status(ProcessStatus::READY);
            unsafe {
                self.apps.push(RefCellWrap::new(process));
            }
            app_id as i32
        } else {
            error!("The app pool now is full, can't add new app");
            return -1;
        }
    }

    pub fn current_app(&mut self) -> RefMut<Process> {
        self.app(self.current)
    }

    pub fn next_app(&mut self) -> RefMut<Process> {
        // When the next api be called, there must be at least one apps in vector
        let next = (self.current + 1) % self.apps.len();
        self.current = next;
        self.app(self.current)
    }

    pub fn run_apps(&self) {

    }
}

#[derive(Copy, Clone)]
pub struct Process {
    pub id: usize,
    pub base_address: usize,
    pub status: ProcessStatus,
    pub ctx: SwitchContext,
}

impl Process {
    pub fn new(id: usize, base_addr: usize) -> Self {
        Process {
            id: id,
            base_address: base_addr,
            status: ProcessStatus::UNINIT,
            ctx: SwitchContext::new_with_restore_addr(
                KERNEL_STACKS[id].push_context(
                    TrapContext::new(base_addr)
                )
            ),
        }
    }

    pub fn set_status(&mut self, status: ProcessStatus) {
        self.status = status;
    }
}

#[derive(Copy, Clone)]
pub enum ProcessStatus {
    UNINIT,
    READY,
    RUNNING,
    EXITED,
}
