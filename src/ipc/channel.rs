// 使用 sem 和 shm 组合成一个通道，实现 File trait，为 app 提供一个更加方便的 IPC 方式
// channel 是一对多，而且同步的，主要用于 request data 的场景，通常由 server 创建

use alloc::vec::Vec;
use alloc::sync::{Weak, Arc};
use spin::mutex::Mutex;
use crate::process::app::Process;
use crate::process::pid::PidHandler;

use super::rcvid::{rcvid_alloc, RcvidHandler};

pub struct Channel {
    server: Weak<Mutex<Process>>,
    msgs: Vec<Arc<Mutex<Msg>>>,
}

impl Channel {
    pub fn new(server: Weak<Mutex<Process>>) -> Self {
        Self { server, msgs: Vec::new() }
    }

    pub fn send(&mut self, client: PidHandler, data: Arc<Vec<u8>>) {
        let msg: Arc<Mutex<Msg>> = Arc::new(Mutex::new(Msg::new(client, data).unwrap()));
        self.msgs.push(msg);
    }

    pub fn recv(&mut self) -> isize {
        self.msgs.iter().position(|m| m.lock().client.0 )
    }
}

pub struct Msg {
    rcvid: RcvidHandler,
    client: PidHandler,
    data: Arc<Vec<u8>>,
}

impl Msg {
    pub fn new(client: PidHandler, data: Arc<Vec<u8>>) -> Option<Self> {
        let rcvid = rcvid_alloc()?;
        Some(Self { rcvid, client, data })
    }
}