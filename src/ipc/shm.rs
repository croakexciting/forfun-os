use core::borrow::Borrow;

use alloc::vec::Vec;
use crate::mm::{
    allocator::{frame_alloc, PhysFrame}, 
    area::Permission, 
    basic::{PhysPage, VirtPage}, 
    MemoryManager
};

pub struct Shm {
    pub users: Vec<usize>,

    _frames: Vec<PhysFrame>,
    ppns: Vec<PhysPage>,
    permission: Permission,
}

impl Shm {
    pub fn new(pn: usize, permission: usize) -> Self {
        let mut p = Permission::from_bits_truncate((permission as u8) << 1);
        p.insert(Permission::U);
        let mut _frames = Vec::with_capacity(pn);
        let mut ppns = Vec::with_capacity(pn);
        for _ in 0..pn {
            let frame = frame_alloc().unwrap();
            ppns.push(frame.ppn);
            _frames.push(frame);
        }

        Self { users: Vec::new(), _frames, ppns, permission: p }
    }

    pub fn map(&mut self, pid: usize, mm: &mut MemoryManager) -> isize {
        self.users.push(pid);
        mm.map_defined(self.ppns.borrow(), self.permission)
    }

    pub fn unmap(&mut self, pid: usize, start_vpn: VirtPage, mm: &mut MemoryManager) -> isize {
        if let Some(index) = self.users.iter().position(|p| *p == pid) {
            self.users.remove(index);
            mm.umap_dyn_area(start_vpn)
        } else {
            -1
        }
    }
}