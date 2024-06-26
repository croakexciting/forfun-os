use lazy_static::*;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;

lazy_static!{
    pub static ref RCVID_ALLOCATOR: Arc<Mutex<IdAllocator>> = {
        Arc::new(
            Mutex::new(IdAllocator::new())
        )
    };

    pub static ref COID_ALLOCATOR: Arc<Mutex<IdAllocator>> = {
        Arc::new(
            Mutex::new(IdAllocator::new())
        )
    };
}

pub struct IdAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl IdAllocator {
    pub fn new() -> Self {
        Self { current: 0, end: 0xFFFF, recycled: Vec::new() }
    }

    // 按照顺序
    pub fn alloc(&mut self) -> Option<usize> {
        if self.current == self.end {
            if let Some(id) = self.recycled.pop() {
                Some(id)
            } else {
                None
            }
        } else {
            self.current += 1;
            Some(self.current - 1)
        }
    }

    #[allow(unused)]
    pub fn dealloc(&mut self, id: usize) {
        if id >= self.current || self.recycled.iter().any(|&v| v == id) {
            // 既不在 recycled 中，也不在未分配的内存范围中
            // do nothing
        }

        self.recycled.push(id);
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct RcvidHandler(pub usize);

impl RcvidHandler {
    pub fn new_with_rcvid(rcvid: usize) -> Self {
        Self(rcvid)
    }
}

impl Drop for RcvidHandler {
    fn drop(&mut self) {
        RCVID_ALLOCATOR.lock().dealloc(self.0);
    }
}

pub fn rcvid_alloc() -> Option<RcvidHandler> {
    let id = RCVID_ALLOCATOR.lock().alloc()?;
    Some(RcvidHandler(id))
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct CoidHandler(pub usize);

impl Drop for CoidHandler {
    fn drop(&mut self) {
        COID_ALLOCATOR.lock().dealloc(self.0);
    }
}

pub fn coid_alloc() -> Option<CoidHandler> {
    let id = COID_ALLOCATOR.lock().alloc()?;
    Some(CoidHandler(id))
}
