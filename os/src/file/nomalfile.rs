use rcore_fs::vfs as rcore_vfs;
use alloc::sync::Arc;
use spin::mutex::Mutex;

use super::{File, FileError, FilePermission};

pub struct NormalFile {
    inode: Arc<dyn rcore_vfs::INode>,
    permission: FilePermission,
    seek: Arc<Mutex<usize>>,
}

impl NormalFile {
    pub fn new(inode: Arc<dyn rcore_vfs::INode>, permission: FilePermission) -> Self {
        Self { inode, permission, seek: Arc::new(Mutex::new(0)) }
    }
}

impl File for NormalFile {
    fn readable(&self) -> bool {
        self.permission.contains(FilePermission::R)
    }

    fn writable(&self) -> bool {
        self.permission.contains(FilePermission::W)
    }

    fn read(&self, buf: &mut crate::mm::area::UserBuffer) -> Result<usize, FileError> {
        let filesize = self.inode.metadata()?.size;
        let seek = *self.seek.lock();
        let mut blk_size = buf.buffer.len();
        if seek + buf.buffer.len() > filesize {
            blk_size = filesize - seek;
        }

        let size = self.inode.read_at(*self.seek.lock(), &mut buf.buffer[..blk_size])?;

        Ok(size)
    }

    fn size(&self) -> Result<usize, FileError> {
        let size = self.inode.metadata()?.size;
        Ok(size)
    }

    fn write(&self, buf: &crate::mm::area::UserBuffer) -> Result<usize, FileError> {
        let size = self.inode.write_at(*self.seek.lock(), &buf.buffer)?;
        Ok(size)
    }

    fn lseek(&self, seek: usize) -> isize {
        let mut seek_ptr = self.seek.lock();
        *seek_ptr = seek;
        seek as isize
    }
}

