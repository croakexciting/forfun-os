#![no_std]
#![no_main]

use ffos_app::syscall::{sys_create_server, sys_exec, sys_fork, sys_lseek, sys_open, sys_read, sys_recv_server, sys_reply_server, sys_write};

#[macro_use]
extern crate ffos_app;

extern crate alloc;
use rcore_fs::{dev::{DevError, Device}, vfs::{FileType, Result}};
use rcore_fs_sfs::SimpleFileSystem;
use spin::mutex::Mutex;
use alloc::{sync::Arc, vec::Vec, vec};
use rcore_fs::vfs::FileSystem;

const CREATE_FILE_SERVER: &'static str = "create_file\0";
const READ_FILE_SERVER: &'static str = "read_file\0"; 

pub struct BlkDevice {
    fd: Mutex<usize>,
}

impl BlkDevice {
    pub fn new() -> Option<Self> {
        let fd = sys_open("qemu-blk\0");
        if fd >= 0 {
            return Some(BlkDevice { fd: Mutex::new(fd as usize) });
        } else {
            return None;
        }
    }
}

impl Device for BlkDevice {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> rcore_fs::dev::Result<usize> {
        let fd = self.fd.lock().clone();
        sys_lseek(fd, offset);
        if sys_read(fd, buf) == (buf.len() as isize) {
            return Ok(buf.len());
        } else {
            Err(DevError)
        }
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> rcore_fs::dev::Result<usize> {
        let fd = self.fd.lock().clone();
        sys_lseek(fd, offset);
        if sys_write(fd, buf) == (buf.len() as isize) {
            return Ok(buf.len());
        } else {
            Err(DevError)
        }
    }

    fn sync(&self) -> rcore_fs::dev::Result<()> {
        Ok(())
    }
}

#[allow(unused)]
fn create_file(name: &str, sfs: Arc<SimpleFileSystem>) -> Result<()> {
    let root = sfs.root_inode();
    let file = root.create(name, FileType::File, 0o777)?;
    sfs.sync()?;
    Ok(())
}

#[allow(unused)]
fn ls(sfs: Arc<SimpleFileSystem>) {
    let root = sfs.root_inode();
    let files = root.list().unwrap();
    for f in files {
        println!("{}", f);
    }
}

fn elf_load_test(sfs: Arc<SimpleFileSystem>, app: &str) -> Result<()> {
    let root = sfs.root_inode();
    let file = root.find(app)?;
    let size = file.metadata().unwrap().size;
    let pid = sys_fork();
    if pid == 0 {
        let mut data: vec::Vec<u8> = vec![0; size];
        file.read_at(0, data.as_mut_slice())?;
        sys_exec(data.as_slice());
    }

    Ok(())
}

fn read_file(sfs: Arc<SimpleFileSystem>, name: &str) -> Result<Vec<u8>> {
    let root = sfs.root_inode();
    let file = root.find(name)?;
    let size = file.metadata().unwrap().size;
    let mut data: vec::Vec<u8> = vec![0; size];
    file.read_at(0, data.as_mut_slice())?;
    Ok(data)
}

#[no_mangle]
fn main() -> i32 {
    let sfs: Arc<SimpleFileSystem> = {
        match SimpleFileSystem::open(Arc::new(BlkDevice::new().unwrap())) {
            Ok(sfs) => {
                sfs
            },
            Err(_) => {
                SimpleFileSystem::create(
                    Arc::new(BlkDevice::new().unwrap()),
                    1024*1024*1024
                ).expect("File System create failed")
            }
        }
    };

    elf_load_test(sfs.clone(), "shell").expect("load shell failed");
    sys_create_server(&CREATE_FILE_SERVER);
    sys_create_server(&READ_FILE_SERVER);

    let mut rcv_buf: vec::Vec<u8> = vec![0; 256];
    let mut rcv_len: usize = 0;
    loop {
        // TODO: 后面可以考虑实现类似 select 的功能
        // TODO: 暂时只考虑支持一级文件系统
        let rcvid = sys_recv_server(
            &CREATE_FILE_SERVER, 
            rcv_buf.as_mut_ptr(), 
            &mut rcv_len, 10);
        if rcvid >= 0 {
            
        }

        let rcvid = sys_recv_server(
            &READ_FILE_SERVER, 
            rcv_buf.as_mut_ptr(), 
            &mut rcv_len, 10);
        if rcvid >= 0 {
            // handler
            let msg = core::str::from_utf8(&rcv_buf.as_slice()[0..rcv_len]).unwrap();
            let data = read_file(sfs.clone(), msg).expect("read file content error");
            sys_reply_server(rcvid as usize, data.as_slice());
        }
    };
}
