// page table manager

use core::borrow::BorrowMut;

use alloc::vec;
use alloc::vec::Vec;
use super::{allocator::{frame_alloc, PhysFrame}, basic::{PTEFlags, PageTableEntry, PhysPage, VirtPage}};

// Every app has it's own page table
pub struct PageTable {
    // 第一级页表的页号，其实就是地址
    root: PhysPage,
    // 存储页表的物理页帧，放在这里只是为了页表实例回收的时候自动将 Frame dealloc
    frames: Vec<PhysFrame>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        Self {
            root: frame.ppn,
            frames: vec![frame],
        }
    }

    // 在过程中，会自动创建树干页表中的页表项，但是不会创建叶子页表中的页表项
    // 之所以这么做，是因为树干页表中的页表项指向的是页表，这在页表类中可以管理
    // 而叶子页表中的页表项指向的是 text, data 等段，这是页表类无法管理的，也无需负责管理
    // 而且还有一个原因是，根据虚拟页号查询时，得到树干页表中的页表项没有意义，因为使用者根本
    // 不想关心你的页表是如何管理的，只需要给他们对应的物理页帧就可以了
    pub fn find_pte(&mut self, vpn: VirtPage) -> Option<&mut PageTableEntry> {
        let idx = vpn.index();
        let mut ppn = self.root;
        for (k, v) in idx.iter().enumerate() {
            let pte = ppn.pte_array()[*v].borrow_mut();
            if k == 2 {
                // 从叶子页表中获得了 PTE，直接返回
                return Some(pte)
            } else {
                if !pte.is_valid() {
                    let frame = frame_alloc().unwrap();
                    // 创建一个树干页表，只需要 valid flag 即可
                    *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                    self.frames.push(frame);
                }
            }
            ppn = pte.ppn();
        }
        None
    }

    // 事实上你可以将虚拟地址看成是 index，用于寻找到对应的 PTE，然后根据物理页帧信息修改 PTE
    pub fn map(&mut self, vpn: VirtPage, ppn: PhysPage, flags: PTEFlags) -> i32 {
        let pte = self.find_pte(vpn).unwrap();
        if pte.is_valid() {
            // already used
            return -1
        }
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
        return 0
    }

    pub fn unmap(&mut self, vpn: VirtPage) -> i32 {
        let pte = self.find_pte(vpn).unwrap();
        if !pte.is_valid() {
            // not used. don't need unmap
            return -1;
        }

        pte.clear();
        return 0;
    }
}