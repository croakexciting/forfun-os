use alloc::vec::Vec;
use lazy_static::*;
use crate::utils::type_extern::RefCellWrap;

use super::basic::*;

// reserve for apps
const KERNEL_ALLOCATOR_START: usize = 0x80150000;
const ALLOCATOR_START: usize = 0x80200000;
const ALLOCATOR_END: usize = 0x807FFFF0;

// This struct locate at memory
pub struct PhysFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl PhysFrameAllocator {
    pub fn new(start: PhysPage, end: PhysPage) -> Self {
        Self { current: start.0, end: end.0, recycled: Vec::new() }
    }

    // 按照顺序
    pub fn alloc(&mut self) -> Option<PhysPage> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }

    pub fn dealloc(&mut self, ppn: PhysPage) {
        if ppn.0 >= self.current || self.recycled.iter().any(|&v| v == ppn.0) {
            // 既不在 recycled 中，也不在未分配的内存范围中
            panic!("Frame ppn={} has not been allocated!", ppn.0);
        }

        self.recycled.push(ppn.0);
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: RefCellWrap<PhysFrameAllocator> = unsafe {
        let startaddr: PhysPage = PhysAddr::from(ALLOCATOR_START).into();
        let endaddr: PhysPage = PhysAddr::from(ALLOCATOR_END - 1).into();

        RefCellWrap::new(PhysFrameAllocator::new(startaddr, endaddr))
    };

    pub static ref ASID_ALLOCATOR: RefCellWrap<AsidAllocator> = unsafe {
        RefCellWrap::new(AsidAllocator::new())
    };

    pub static ref KERNEL_FRAME_ALLOCATOR: RefCellWrap<PhysFrameAllocator> = unsafe {
        let start_ppn: PhysPage = PhysAddr::from(KERNEL_ALLOCATOR_START).into();
        let end_ppn: PhysPage = PhysAddr::from(ALLOCATOR_START - 1).into();

        RefCellWrap::new(PhysFrameAllocator::new(start_ppn, end_ppn))
    };
}

pub struct PhysFrame {
    pub ppn: PhysPage,
}

impl PhysFrame {
    pub fn new(ppn: PhysPage) -> Self {
        // clean physcal frame
        let addr: PhysAddr = ppn.into();
        unsafe { core::slice::from_raw_parts_mut(addr.0 as *mut usize, 512).fill(0) };
        Self { ppn }
    }
}

impl Drop for PhysFrame {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.exclusive_access().dealloc(self.ppn);
    }
}

pub fn frame_alloc() -> Option<PhysFrame> {
    FRAME_ALLOCATOR.exclusive_access().alloc().map(|p| PhysFrame::new(p))
}

pub fn kernel_frame_alloc() -> Option<PhysFrame> {
    KERNEL_FRAME_ALLOCATOR.exclusive_access().alloc().map(|p| PhysFrame::new(p))
}

pub struct AsidAllocator {
    current: u16,
    end: u16,
    recycled: Vec<u16>,
}

impl AsidAllocator {
    pub fn new() -> Self {
        Self { current: 0, end: 0xFFFF, recycled: Vec::new() }
    }

    // 按照顺序
    pub fn alloc(&mut self) -> Option<u16> {
        if let Some(asid) = self.recycled.pop() {
            Some(asid)
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some(self.current - 1)
        }
    }

    pub fn dealloc(&mut self, asid: u16) {
        if asid >= self.current || self.recycled.iter().any(|&v| v == asid) {
            // 既不在 recycled 中，也不在未分配的内存范围中
            panic!("Frame ppn={} has not been allocated!", asid);
        }

        self.recycled.push(asid);
    }
}
