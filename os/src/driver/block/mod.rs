use alloc::string::String;
use alloc::{sync::Arc, vec};
use spin::mutex::Mutex;
use rcore_fs;

pub mod qemu_blk;

pub trait BlockDevice: Send + Sync {
    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> Result<usize, String>;
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result<usize, String>;
    fn block_size_log2(&self) -> u8;
}

pub struct BlockIter {
    pub start: usize,
    pub end: usize,
    pub block_size_log2: u8,
}

pub struct SubBlk {
    pub id: usize,
    pub start: usize,
    pub end: usize,
    pub block_size_log2: u8,
}

impl SubBlk {
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_full(&self) -> bool {
        self.len() == (1usize << self.block_size_log2)
    }

    // pub fn offset_start(&self) -> usize {
    //     (self.id << self.block_size_log2) + self.start
    // }

    // pub fn offset_end(&self) -> usize {
    //     (self.id << self.block_size_log2) + self.end
    // }
}

impl Iterator for BlockIter {
    type Item = SubBlk;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        let blk_size = 1usize << self.block_size_log2;
        let block = self.start / blk_size;
        let begin = self.start % blk_size;
        let end = if block == self.end / blk_size {
            self.end % blk_size
        } else {
            blk_size
        };
        self.start += end - begin;
        Some(SubBlk { 
            id: block, 
            start: begin, 
            end, 
            block_size_log2: self.block_size_log2 
        })
    }
}

pub struct BlkDeviceForFs {
    device: Arc<Mutex<dyn BlockDevice>>,
}

impl BlkDeviceForFs {
    pub fn new(device: Arc<Mutex<dyn BlockDevice>>) -> Self {
        Self { device }
    }
}

impl rcore_fs::dev::Device for BlkDeviceForFs {
    fn read_at(&self, seek: usize, buf: &mut [u8]) -> rcore_fs::dev::Result<usize> {
        let block_iter = BlockIter {
            start: seek,
            end: seek + buf.len(),
            block_size_log2: self.device.lock().block_size_log2()
        };

        let mut offset: usize = 0;

        for sub in block_iter {
            let blk_size = 1usize << sub.block_size_log2;
            if sub.is_full() {
                let slice = &mut buf[offset..(offset + blk_size)];
                if let Err(e) = self.device.lock().read_block(sub.id, slice) {
                    println!("[kernel] read block failed: {}", e.as_str());
                    return Err(rcore_fs::dev::DevError)
                }
                offset += blk_size;
            } else {
                // if the size less than block size, need copy
                let mut temp: vec::Vec<u8> = vec![0; blk_size];
                let dst = &mut buf[offset..(offset + sub.len())];
                if let Err(e) = self.device.lock().read_block(sub.id, temp.as_mut_slice()) {
                    println!("[kernel] read block failed: {}", e.as_str());
                    return Err(rcore_fs::dev::DevError)
                }
                let src = &temp.as_slice()[sub.start..sub.end];
                dst.copy_from_slice(src);
                offset += sub.len()
            }
        }

        Ok(offset)
    }

    fn write_at(&self, seek: usize, buf: &[u8]) -> rcore_fs::dev::Result<usize> {
        let block_iter = BlockIter {
            start: seek,
            end: seek + buf.len(),
            block_size_log2: self.device.lock().block_size_log2()
        };

        let mut offset: usize = 0;

        for sub in block_iter {
            let blk_size = 1usize << sub.block_size_log2;
            if sub.is_full() {
                let slice = &buf[offset..(offset + sub.len())];
                if let Err(e) = self.device.lock().write_block(sub.id, slice) {
                    println!("[kernel] write block failed: {}", e.as_str());
                    return Err(rcore_fs::dev::DevError)
                }
                offset += sub.len();
            } else {
                let mut temp: vec::Vec<u8> = vec![0; blk_size];
                let src = &buf[offset..(offset + sub.len())];
                let dst = &mut temp.as_mut_slice()[sub.start..sub.end];
                dst.copy_from_slice(src);
                if let Err(e) = self.device.lock().write_block(sub.id, temp.as_mut_slice()) {
                    println!("[kernel] write block failed: {}", e.as_str());
                    return Err(rcore_fs::dev::DevError)
                }
                offset += sub.len();
            }
        }

        Ok(offset)
    }

    fn sync(&self) -> rcore_fs::dev::Result<()> {
        // since the device inside a mutex lock, just return Ok
        Ok(())
    }
}