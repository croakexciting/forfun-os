use crate::config::*;

use alloc::vec;
use alloc::vec::Vec;
use log::error;

use super::context::SwitchContext;


pub struct AppManager {
    current: usize,
    apps: Vec<Process>,
}

impl AppManager {
    pub fn new() -> Self {
        AppManager {
            current: 0,
            apps: vec![],
        }
    }

    // pub fn app(&self)
    pub fn app(&self, id: usize) -> Process {
        self.apps[id].clone()
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

    pub fn current_app(&mut self) -> Process {
        self.apps[self.current].clone()
    }

    pub fn next_app(&mut self) -> Process {
        // When the next api be called, there must be at least one apps in vector
        let next = (self.current + 1) % self.apps.len();
        self.current = next;
        self.apps[next].clone()
    }

    pub fn set_status(&mut self, id: usize, status: ProcessStatus) {
        self.apps[id].set_status(status);
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
            ctx: SwitchContext::new(0, 0),
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
