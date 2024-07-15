use alloc::sync::Weak;
use alloc::vec::Vec;
use spin::mutex::Mutex;

use crate::process::app::Process;

pub struct Semaphore {
    count: usize,
    wait_list: Vec<Weak<Mutex<Process>>>
}

impl Semaphore {
    pub fn new() -> Self {
        Self { count: 0, wait_list: Vec::new() }
    }

    pub fn wait(&mut self, p: Weak<Mutex<Process>>) {
        self.wait_list.push(p);
    }

    pub fn raise(&mut self) -> Option<Weak<Mutex<Process>>> {
        self.count += 1;
        let proc = self.wait_list.pop()?;
        self.count -= 1;
        Some(proc)
    }

    pub fn check(&mut self) -> Option<Weak<Mutex<Process>>> {
        if self.count > 0 {
            self.wait_list.pop()
        } else {
            None
        }
    }
}