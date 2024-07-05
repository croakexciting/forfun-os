use alloc::string::String;
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