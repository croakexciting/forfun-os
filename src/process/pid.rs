use lazy_static::*;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;

use crate::utils::type_extern::RefCellWrap;

// 由于我们的内核目前设计就是单线程，因此可以使用低效率的自旋锁
// 它其实和之前使用的 RefCell 效率是相同的，无非一个是报错，另一个是傻傻等待
// 后期考虑效率，需要根据 CPU 架构使用硬件支持的锁
lazy_static!{
    pub static ref PID_ALLOCATOR: Arc<Mutex<PidAllocator>> = unsafe {
        Arc::new(
            Mutex::new(PidAllocator::new())
        )
    };
}

pub struct PidAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl PidAllocator {
    pub fn new() -> Self {
        Self { current: 0, end: 0xFFFF, recycled: Vec::new() }
    }

    // 按照顺序
    pub fn alloc(&mut self) -> Option<PidHandler> {

        if self.current == self.end {
            if let Some(pid) = self.recycled.pop() {
                Some(PidHandler(pid))
            } else {
                None
            }
        } else {
            self.current += 1;
            Some(PidHandler(self.current - 1))
        }
    }

    #[allow(unused)]
    pub fn dealloc(&mut self, pid: usize) {
        if pid >= self.current || self.recycled.iter().any(|&v| v == pid) {
            // 既不在 recycled 中，也不在未分配的内存范围中
            panic!("Pid={} has not been allocated!", pid);
        }

        self.recycled.push(pid);
    }
}

#[derive(Clone)]
pub struct PidHandler(pub usize);

impl Drop for PidHandler {
    fn drop(&mut self) {
        PID_ALLOCATOR.lock().dealloc(self.0);
    }
}

pub fn alloc() -> Option<PidHandler> {
    PID_ALLOCATOR.lock().alloc()
}
