## 1 Introduction

This page will introduce the driver and filesystem in Forfun OS.

At the beginning, I want to implement a micro kernel, just like qnx or seL4. Actually, I finished it, ref to this [version](https://github.com/croakexciting/forfun-os/commit/eb4afee9d83d605cf2781434c291365061d2d5d8). But put driver and filesystem is not easy to develop, I change my mind and put drivers and filesystem inside kernel space.

Thanks to rcore community, I can use rcore-fs instead of develop filesystem from scratch. I just need to implement the block device read write interface. Besides, I create and initialize a filesystem globle instance in kernel. That's all, very simple.

## 2 Filesystem

I have create a file trait, any object which implement this trait will be managed as a file. This is the **anything is file** design which proposed by UNIX. 

Process (TCB) instance contains a file descriptor set, which ref to the files opend by process. User space program use write, read, open, close syscalls to communicate with file.

So we design a struct named with NormalFile and implement file trait for it. The core function is read and write. Let's take a look at them

```
pub struct NormalFile {
    inode: Arc<dyn rcore_vfs::INode>,
    permission: FilePermission,
    seek: Arc<Mutex<usize>>,
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

    fn write(&self, buf: &crate::mm::area::UserBuffer) -> Result<usize, FileError> {
        let size = self.inode.write_at(*self.seek.lock(), &buf.buffer)?;
        Ok(size)
    }
    ...
}
```

Inode is the unit of filesystem, usually can represent a file.

The read and write function will call inode write and read inside, rcore-fs implement the filesystem middleware. Finally, call the block device driver to write or read hardware data.


Besides, kernel create a global filesystem instance and initialize it, process open files through it. Now we choose to use sfs filesystem, developed by rcore.

Because the filesystem type is the rcore_fs vfs interface, kernel can conveniently switch filesystem.

```
lazy_static! {
    pub static ref FILESYSTEM: RefCellWrap<Filesystem> = unsafe {
        RefCellWrap::new(Filesystem::new())
    };
}

pub struct Filesystem {
    inner: Option<Arc<dyn rcore_fs::vfs::FileSystem>>
}
```

Now, kernel only support one filesystem instance, and can't mount dynamically.

## 3 Driver

Now, Forfun OS driver design is very simple, we define a trait for each type of driver, and implement these trait in specific driver code. For example, the block driver trait and virtio block class show as below.

```
# block trait
pub trait BlockDevice: Send + Sync {
    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> Result<usize, String>;
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result<usize, String>;
    fn block_size_log2(&self) -> u8;
}

# specific driver
pub struct QemuBlk {
    device: VirtIOBlk<HalImpl, MmioTransport>,
    block_size_log2: u8,
}

impl QemuBlk {
    pub fn new(addr: usize) -> Self {
        Self { device: init_blk(addr).unwrap(), block_size_log2: 9 }
    }
}

impl BlockDevice for QemuBlk {
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result<usize, String> {
        match self.device.write_blocks(block_id, buf) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(e.to_string())
        }
    }

    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> Result<usize, String> {
        match self.device.read_blocks(block_id, buf) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(e.to_string())
        }
    }

    fn block_size_log2(&self) -> u8 {
        self.block_size_log2
    }
}
```

We implement the rcore_fs::dev::Device for BlockDevice trait, so we don't need to implement this trait for every block driver. Specific block driver only need to implement BlockDevice trait. Thus block device provide a unified interface for upper module, easy to use and manage.

Drivers will initialize by board_init function, because of each platform has different device base address.

## 4 Conclusion

Forfun OS implement a very simple filesystem function and driver architect. We still have a-lot-of work, such as

- Hardware interrupt handling mechanism
- Multiple filesystem access

We will add those functions later.

Next chapter, I want't to show you the arch and board relative functions.