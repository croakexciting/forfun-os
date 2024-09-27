use lazy_static::*;
use alloc::sync::Arc;
use rcore_fs_sfs::SimpleFileSystem;

use crate::{driver::block::BlkDeviceForFs, utils::type_extern::RefCellWrap};

use super::{nomalfile::NormalFile, File};

lazy_static! {
    pub static ref FILESYSTEM: RefCellWrap<Filesystem> = unsafe {
        RefCellWrap::new(Filesystem::new())
    };
}

pub struct Filesystem {
    inner: Option<Arc<dyn rcore_fs::vfs::FileSystem>>
}

impl Filesystem {
    pub fn new() -> Self {
        Self { inner: None }
    }

    pub fn set_sfs(&mut self, device: BlkDeviceForFs) -> isize {
        if let Ok(sfs) = SimpleFileSystem::open(Arc::new(device)) {
            self.inner = Some(sfs);
            return 0;
        } else {
            return -1
        }
    }

    pub fn open(&mut self, name: &str) -> Option<Arc<dyn File>> {
        if let Some(fs) = self.inner.clone() {
            match fs.root_inode().find(name) {
                Ok(inode) => {
                    return Some(Arc::new(NormalFile::new(inode, super::FilePermission::all())));
                }
                Err(e) => {
                    println!("[kernel] Find file failed: {}", e);
                }
            }
        }

        return None;
    }
}

