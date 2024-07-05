use alloc::{sync::Arc, vec};
use crate::driver::block::{qemu_blk::QemuBlk, BlockDevice, BlockIter};
use spin::mutex::Mutex;

use crate::file::File;

pub struct QemuBlkFile {
    device: Arc<Mutex<dyn BlockDevice>>,
    seek: Arc<Mutex<usize>>,
}

impl QemuBlkFile {
    pub fn new() -> Self {
        Self { 
            device: Arc::new(Mutex::new(QemuBlk::new())), 
            seek: Arc::new(Mutex::new(0)),
        }
    }

    pub fn seek(&self) -> usize {
        let seek = self.seek.lock().clone();
        seek
    }
}

impl File for QemuBlkFile {
    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    fn read(&self, buf: &mut crate::mm::area::UserBuffer) -> isize {
        let block_iter = BlockIter {
            start: self.seek(),
            end: self.seek() + buf.buffer.len(),
            block_size_log2: self.device.lock().block_size_log2(),
        };

        let mut offset: usize = 0;

        for sub in block_iter {
            let blk_size = 1usize << sub.block_size_log2;
            if sub.is_full() {
                let slice = &mut buf.buffer[offset..(offset + blk_size)];
                if let Err(e) = self.device.lock().read_block(sub.id, slice) {
                    println!("[kernel] read block failed: {}", e.as_str());
                    return -1;
                }
                offset += blk_size;
            } else {
                // 对于不满足一个块的大小，需要拷贝
                let mut temp: vec::Vec<u8> = vec![0; blk_size];
                let dst = &mut buf.buffer[offset..(offset + sub.len())];
                if let Err(e) = self.device.lock().read_block(sub.id, temp.as_mut_slice()) {
                    println!("[kernel] read block failed: {}", e.as_str());
                    return -1;
                }
                let src = &temp.as_slice()[sub.start..sub.end];
                dst.copy_from_slice(src);
                offset += sub.len()
            }
        }

        offset as isize

    }

    fn write(&self, buf: &crate::mm::area::UserBuffer) -> isize {
        let block_iter = BlockIter {
            start: self.seek(),
            end: self.seek() + buf.buffer.len(),
            block_size_log2: 9
        };

        let mut offset: usize = 0;

        for sub in block_iter {
            let blk_size = 1usize << sub.block_size_log2;
            if sub.is_full() {
                let slice = &buf.buffer[offset..(offset + sub.len())];
                if let Err(e) = self.device.lock().write_block(sub.id, slice) {
                    println!("[kernel] write block failed: {}", e.as_str());
                    return -1;
                }
                offset += sub.len();
            } else {
                let mut temp: vec::Vec<u8> = vec![0; blk_size];
                let src = &buf.buffer[offset..(offset + sub.len())];
                let dst = &mut temp.as_mut_slice()[sub.start..sub.end];
                dst.copy_from_slice(src);
                if let Err(e) = self.device.lock().write_block(sub.id, temp.as_mut_slice()) {
                    println!("[kernel] write block failed: {}", e.as_str());
                    return -1;
                }
                offset += sub.len();
            }
        }

        offset as isize
    }

    fn lseek(&self, seek: usize) -> isize {
        let mut seek_ptr = self.seek.lock();
        *seek_ptr = seek;
        seek as isize
    }
}