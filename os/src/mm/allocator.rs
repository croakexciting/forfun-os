use alloc::vec::Vec;
use lazy_static::*;
use crate::utils::type_extern::RefCellWrap;
use crate::arch::memory::page::*;
use crate::board::peri::memory::{
    KERNEL_ALLOCATOR_START,
    ALLOCATOR_START,
    ALLOCATOR_END,
};

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

    pub static ref KERNEL_FRAME_ALLOCATOR: RefCellWrap<PhysFrameAllocator> = unsafe {
        let start_ppn: PhysPage = PhysAddr::from(KERNEL_ALLOCATOR_START).into();
        let end_ppn: PhysPage = PhysAddr::from(ALLOCATOR_START - 1).into();

        RefCellWrap::new(PhysFrameAllocator::new(start_ppn, end_ppn))
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

impl Drop for PhysFrame {
    fn drop(&mut self) {
        let kernel_end_ppn: PhysPage = PhysAddr::from(ALLOCATOR_START).into();
        if self.ppn < kernel_end_ppn {
            KERNEL_FRAME_ALLOCATOR.exclusive_access().dealloc(self.ppn)
        } else {
            FRAME_ALLOCATOR.exclusive_access().dealloc(self.ppn);
        }
    }
}

pub fn frame_alloc() -> Option<PhysFrame> {
    FRAME_ALLOCATOR.exclusive_access().alloc()
}

pub fn kernel_frame_alloc() -> Option<PhysFrame> {
    let frame = KERNEL_FRAME_ALLOCATOR.exclusive_access().alloc()?;
    kernel_page_phys_to_virt(frame.ppn).clear_page();
    Some(frame)
}

pub struct AsidAllocator {
    current: u16,
    end: u16,
    recycled: Vec<u16>,
}

impl AsidAllocator {
    pub fn new() -> Self {
        // asid 0 reserved for kernel
        Self { current: 1, end: 0xFFFF, recycled: Vec::new() }
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
