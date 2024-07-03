use alloc::vec::Vec;
use lazy_static::*;
use crate::utils::type_extern::RefCellWrap;

use super::{basic::*, buddy::BuddyAllocator};

// reserve for apps
const KERNEL_ALLOCATOR_START: usize = 0x80380000;
const ALLOCATOR_START: usize = 0x80400000;
const ALLOCATOR_END: usize = 0x80800000;

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
    pub fn alloc(&mut self) -> Option<PhysFrame> {
        if self.current == self.end {
            if let Some(ppn) = self.recycled.pop() {
                Some(PhysFrame::new(ppn.into()))
            } else {
                None
            }
        } else {
            let ppn: PhysPage = (self.current).into();
            self.current += 1;
            Some(PhysFrame::new(ppn))
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

    pub static ref KERNEL_FRAME_ALLOCATOR: RefCellWrap<BuddyAllocator> = unsafe {
        let start_ppn: PhysPage = PhysAddr::from(KERNEL_ALLOCATOR_START).into();
        let pn = (ALLOCATOR_START - KERNEL_ALLOCATOR_START) / PAGE_SIZE;

        RefCellWrap::new(BuddyAllocator::new(5, start_ppn.0.into(), pn))
    };
}
#[derive(Clone)]
pub struct PhysFrame {
    pub ppn: PhysPage,
}

impl PhysFrame {
    pub fn new(ppn: PhysPage) -> Self {
        // clean physcal frame
        // 空间清理的工作需要交给调用者来执行
        // unsafe { core::slice::from_raw_parts_mut(addr.0 as *mut usize, 512).fill(0) };
        Self { ppn }
    }
}

pub struct PhysFrames {
    pub ppn: PhysPage,
    pub pn: usize,
}

impl PhysFrames {
    pub fn new(ppn: PhysPage, pn: usize) -> Self {
        Self { ppn, pn}
    }
}

impl Drop for PhysFrame {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.exclusive_access().dealloc(self.ppn);
    }
}

// impl Drop for PhysFrames {
//     fn drop(&mut self) {
//         KERNEL_FRAME_ALLOCATOR.exclusive_access().dealloc(self.ppn.0.into(), self.pn)
//     }
// }

pub fn frame_alloc() -> Option<PhysFrame> {
    FRAME_ALLOCATOR.exclusive_access().alloc()
}

pub fn frame_dealloc(ppn: PhysPage) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn)
}

pub fn kernel_frame_alloc(pn: usize) -> Option<PhysFrames> {
    let start = KERNEL_FRAME_ALLOCATOR.exclusive_access().alloc(pn)?;
    let pfs = PhysFrames::new(start.0.into(), pn);
    Some(pfs)
}

pub fn kernel_frame_dealloc(ppn: PhysPage, pn: usize) {
    KERNEL_FRAME_ALLOCATOR.exclusive_access().dealloc(ppn.0.into(), pn);
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
    pub fn alloc(&mut self) -> Option<AisdHandler> {
        if let Some(asid) = self.recycled.pop() {
            Some(AisdHandler(asid))
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some(AisdHandler(self.current - 1))
        }
    }

    #[allow(unused)]
    pub fn dealloc(&mut self, asid: u16) {
        if asid >= self.current || self.recycled.iter().any(|&v| v == asid) {
            // 既不在 recycled 中，也不在未分配的内存范围中
            panic!("Frame ppn={} has not been allocated!", asid);
        }

        self.recycled.push(asid);
    }
}

pub struct AisdHandler(pub u16);

impl Drop for AisdHandler {
    fn drop(&mut self) {
        ASID_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}

pub fn asid_alloc() -> Option<AisdHandler> {
    ASID_ALLOCATOR.exclusive_access().alloc()
}
