use lazy_static::*;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;

lazy_static!{
    pub static ref RCVID_ALLOCATOR: Arc<Mutex<RcvidAllocator>> = {
        Arc::new(
            Mutex::new(RcvidAllocator::new())
        )
    };
}

pub struct RcvidAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl RcvidAllocator {
    pub fn new() -> Self {
        Self { current: 0, end: 0xFFFF, recycled: Vec::new() }
    }

    // 按照顺序
    pub fn alloc(&mut self) -> Option<RcvidHandler> {
        if let Some(rcvid) = self.recycled.pop() {
            Some(RcvidHandler(rcvid))
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some(RcvidHandler(self.current - 1))
        }
    }

    #[allow(unused)]
    pub fn dealloc(&mut self, rcvid: usize) {
        if rcvid >= self.current || self.recycled.iter().any(|&v| v == rcvid) {
            // 既不在 recycled 中，也不在未分配的内存范围中
            panic!("rcvid={} has not been allocated!", rcvid);
        }

        self.recycled.push(rcvid);
    }
}

#[derive(Clone)]
pub struct RcvidHandler(pub usize);

impl Drop for RcvidHandler {
    fn drop(&mut self) {
        RCVID_ALLOCATOR.lock().dealloc(self.0);
    }
}

pub fn rcvid_alloc() -> Option<RcvidHandler> {
    RCVID_ALLOCATOR.lock().alloc()
}
