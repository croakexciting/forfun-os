use core::borrow::BorrowMut;
use core::cell::RefMut;

use crate::process::switch::__switch;
use crate::config::*;
use crate::trap::context::TrapContext;
use crate::utils::timer::nanoseconds;
use crate::utils::type_extern::RefCellWrap;

use alloc::vec;
use alloc::vec::Vec;
use log::error;

use super::context::SwitchContext;
use super::KERNEL_STACKS;


pub struct AppManagerInner {
    started: bool,
    current: usize,
    apps: Vec<Process>,
    idle_ctx: SwitchContext,
}

impl AppManagerInner {
    pub fn new() -> Self {
        AppManagerInner {
            started: false,
            current: 0,
            apps: vec![],
            // idle process is a unstop loop process
            idle_ctx: SwitchContext::new(0, 0),
        }
    }

    // pub fn app(&self)
    fn app(&mut self, id: usize) -> &mut Process {
        self.apps[id].borrow_mut()
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
            self.apps.push(process);
            app_id as i32
        } else {
            error!("The app pool now is full, can't add new app");
            return -1;
        }
    }

    fn current_app(&mut self) -> &mut Process {
        self.app(self.current)
    }

    fn next_app(&mut self) -> &mut Process {
        assert!(self.apps.len() > 0, "The app vector is empty!!!");
        if self.started {
            // When the next api be called, there must be at least one apps in vector
            let next = (self.current + 1) % self.apps.len();
            self.current = next;
            self.app(next)
        } else {
            self.started = true;
            self.app(0)
        }
    }
}

#[derive(Copy, Clone)]
pub struct Process {
    pub id: usize,
    pub status: ProcessStatus,
    pub ctx: SwitchContext,
}

impl Process {
    pub fn new(id: usize, base_addr: usize) -> Self {
        Process {
            id: id,
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

    pub fn ctx_ptr(&mut self) -> *mut SwitchContext {
        self.ctx.borrow_mut() as *mut _
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ProcessStatus {
    UNINIT,
    READY,
    RUNNING,
    // sleep status with start and duration timestamp(ns) 
    SLEEP(usize, usize),
    EXITED,
}

pub struct AppManager {
    inner: RefCellWrap<AppManagerInner>,
}

impl AppManager {
    pub unsafe fn new() -> Self {
        Self {
            inner: RefCellWrap::new(AppManagerInner::new())
        }
    }

    pub fn inner_access(&self) -> RefMut<'_, AppManagerInner> {
        self.inner.exclusive_access()
    }

    pub fn run_apps(&self) -> ! {
        use ProcessStatus::*;
        loop {
            let mut inner = self.inner_access();
            let idle_ctx = inner.idle_ctx();
            let next = inner.next_app();
            let next_ctx_ptr = next.ctx_ptr();
            match next.status {
                READY => unsafe {
                    next.set_status(RUNNING);
                    drop(inner);
                    __switch(idle_ctx, next_ctx_ptr);
                },
                RUNNING => unsafe {
                    drop(inner);
                    __switch(idle_ctx, next_ctx_ptr);
                }
                SLEEP(a, b) => unsafe {
                    if a + b < nanoseconds() {
                        next.set_status(RUNNING);
                        drop(inner);
                        __switch(idle_ctx, next_ctx_ptr);
                    } else {
                        // println!("[kernel] a+b is {}, current is {}", (a+b), nanoseconds());
                        continue;
                    }
                }
                _ => {
                    continue;
                },
            }
        }
    }

    pub fn back_to_idle(&self) {
        let mut inner = self.inner_access();
        let idle_ctx = inner.idle_ctx();
        let current_ctx_ptr = inner.current_app().ctx_ptr();
        drop(inner);
        unsafe {
            __switch(current_ctx_ptr, idle_ctx);   
        }
    }

    pub fn sleep(&self, duration: usize) {
        let mut inner = self.inner_access();
        let idle_ctx = inner.idle_ctx();
        let current_app = inner.current_app();
        
        // set current time and sleep time
        current_app.set_status(ProcessStatus::SLEEP(nanoseconds(), duration));
        let current_ctx_ptr: *mut SwitchContext = current_app.ctx_ptr();
        drop(inner);
        unsafe {
            __switch(current_ctx_ptr, idle_ctx);
        }
    }

    pub fn exit(&self, exit_code: i32) -> ! {
        let mut inner = self.inner_access();
        let idle_ctx = inner.idle_ctx();
        let current_ctx_ptr = inner.current_app().ctx_ptr();
        inner.current_app().set_status(ProcessStatus::EXITED);
        // println!("[kernel] Application {} exited with code {}", inner.current_app().id, exit_code);
        drop(inner);
        unsafe {
            __switch(current_ctx_ptr, idle_ctx);
            unreachable!()
        }
    }

    pub fn create_app(&self, base_addr: usize) -> i32 {
        let mut inner = self.inner_access();
        inner.create_app(base_addr)
    }
}