// page table manager

use core::borrow::BorrowMut;

use alloc::vec;
use alloc::vec::Vec;
use super::{
    allocator::{kernel_frame_alloc, PhysFrame}, 
    basic::{PTEFlags, PageTableEntry, PhysAddr, PhysPage, VirtAddr, VirtPage}
};

// Every app has it's own page table
pub struct PageTable {
    // 第一级页表的页号，其实就是地址
    root: PhysPage,
    // 存储页表的物理页帧，放在这里只是为了页表实例回收的时候自动将 Frame dealloc
    frames: Vec<PhysFrame>,

    index: usize,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = kernel_frame_alloc().unwrap();
        let ppn = frame.ppn.clone();
        let mut frames: Vec<PhysFrame> = Vec::new();
        frames.reserve(8);
        frames.push(frame);
        Self {
            root: ppn,
            frames,
            index: 0,
        }
    }

    #[allow(unused)]
    pub fn from_ppn(ppn: usize) -> Self {
        Self {
            root: ppn.into(),
            frames: Vec::new(),
            index: 0,
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
            // TODO: 需要考虑下如何在虚地址模式下访问页表
            let pte = ppn.pte_array()[*v].borrow_mut();
            if k == 2 {
                // 从叶子页表中获得了 PTE，直接返回
                return Some(pte)
            } else {
                if !pte.is_valid() {
                    let frame = kernel_frame_alloc().unwrap();
                    // 创建一个树干页表，只需要 valid flag 即可
                    *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                    self.frames.push(frame);
                }
            }
            ppn = pte.ppn();
        }
        None
    }

    pub fn find_valid_pte(&mut self, vpn: VirtPage) -> Option<PageTableEntry> {
        let pte = self.find_pte(vpn).unwrap();
        if pte.is_valid() {
            return Some(pte.clone());
        }

        None
    }

    // 事实上你可以将虚拟地址看成是 index，用于寻找到对应的 PTE，然后根据物理页帧信息修改 PTE
    pub fn map(&mut self, vpn: VirtPage, ppn: PhysPage, flags: PTEFlags) -> Option<PageTableEntry> {
        let pte = self.find_pte(vpn).unwrap();
        if pte.is_valid() {
            // already used
            return None
        }
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
        return Some(*pte)
    }

    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPage) -> i32 {
        let pte = self.find_pte(vpn).unwrap();
        if !pte.is_valid() {
            // not used. don't need unmap
            return -1;
        }

        pte.clear();
        return 0;
    }
    
    pub fn remap(&mut self, vpn: VirtPage, ppn: PhysPage, flags: PTEFlags) -> Option<PageTableEntry> {
        if self.unmap(vpn) < 0 {
            return None
        }

        self.map(vpn, ppn, flags)
    }


    pub fn root_ppn(&self) -> PhysPage {
        self.root
    }

    #[allow(unused)]
    pub fn translate(&mut self, va: VirtAddr) -> Option<PhysAddr> {
        let vp = VirtPage::from(va);
        if let Some(pte) = self.find_pte(vp) {
            let pa = pte.ppn().0 << 12 | (va.0 & ((1<<12) - 1));
            return  Some(pa.into());
        }
        None
    }
   
    // find ceil 用于找到内存范围集合上限对应的物理地址
    // 其实这个地址是用不上的，比如 0x8000 是上限，但是只有 0x7FFF 是实际使用到的地址
    // 如果此时 0x7FFF 对应的 0x107FFF，那么 0x8000 就需要对应 0x108000，即使 0x8000 这个虚拟地址是没有被分配的
    pub fn translate_ceil(&mut self, ceil_va: VirtAddr) -> Option<PhysAddr> {
        let pa = self.translate(ceil_va.reduce(1))?;
        Some(pa.add(1))
    }

    // pub fn fork(&mut self, parent: &mut Self) {
    //     self.index = 0;
    //     for (k, v) in parent {
    //         v.clear_flag(PTEFlags::W);
    //         self.map(k.into(), v.ppn(), v.flags().unwrap());
    //     }
    // }

    pub fn kmap(&mut self, pa: PhysAddr) -> Option<PageTableEntry> {
        let va = VirtAddr::from(pa.0);
        self.map(va.into(), pa.into(), PTEFlags::V | PTEFlags::W | PTEFlags::R)
    }

    pub fn kunmap(&mut self, pa: PhysAddr) -> i32 {
        let va = VirtAddr::from(pa.0);
        self.unmap(va.into())
    }
}

impl Iterator for PageTable {
    type Item = (usize, &'static mut PageTableEntry);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < (1 << 27) {
            let idx = VirtPage::from(self.index).index();
            let mut ppn = self.root;
            for (k, v) in idx.iter().enumerate() {
                let pte = ppn.pte_array()[*v].borrow_mut();
                if pte.is_valid() {
                    if k == 2 {
                        return Some((self.index, pte));
                    } else {
                        ppn = pte.ppn();
                        continue;
                    }
                } else {
                    self.index += 512 ^ (2-k);
                    continue;
                }
            }
        }

        None
    }
}

#[allow(unused)]
pub fn translate(ppn: usize, va: VirtAddr) -> usize {
    let mut pt = PageTable::from_ppn(ppn);
    if let Some(pa) = pt.translate(va) {
        return pa.0;
    }

    0
}