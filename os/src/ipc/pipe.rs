use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use spin::mutex::Mutex;

use crate::mm::area::UserBuffer;

use crate::file::File;

pub struct Pipe {
    readable: bool,
    writable: bool,
    buffer: Arc<Mutex<RingBuffer>>
}

impl Pipe {
    // 直接创造出一对 File (read, write)
    pub fn new(buffer_size: usize) -> (Arc<Self>, Arc<Self>) {
        let buffer = Arc::new(Mutex::new(RingBuffer::new(buffer_size)));
        let read_end = Self {readable: true, writable: true, buffer: 
            buffer.clone()};
        let write_end = Self {readable: false, writable: true, buffer};
        (Arc::new(read_end), Arc::new(write_end))
    }
}

impl File for Pipe {
    fn readable(&self) -> bool {
        self.readable
    }

    fn writable(&self) -> bool {
        self.writable
    }

    fn read(&self, buf: &mut UserBuffer) -> isize {
        self.buffer.lock().read(buf.buffer) as isize
    }

    fn write(&self, buf: &UserBuffer) -> isize {
        self.buffer.lock().write(buf.buffer) as isize
    }

    fn lseek(&self, offset: usize) -> isize {
        offset as isize
    }
}

struct RingBuffer {
    buffer: Vec<u8>,
    head: usize,
    tail: usize,
    empty: bool,
    full: bool
}

impl RingBuffer {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0; size],
            head: 0,
            tail: 0,
            full: false,
            empty: true,
        }
    }

    fn write(&mut self, data: &[u8]) -> usize {
        let mut written = 0;
        for &b in data {
            if self.full {
                break;
            }
            
            self.buffer[self.head] = b;
            self.head = (self.head + 1) % self.buffer.len();
            self.empty = false;
            if self.head == self.tail {
                self.full = true;
            }
            written += 1;
        }

        written
    }

    fn read(&mut self, data: &mut [u8]) -> usize {
        let mut read = 0;
        for b in data.iter_mut() {
            if self.empty {
                break;
            }

            *b = self.buffer[self.tail];
            self.tail = (self.tail + 1) % self.buffer.len();
            self.full = false;
            if self.head == self.tail {
                self.empty = true;
            }
            read += 1;
        }

        read
    }
}