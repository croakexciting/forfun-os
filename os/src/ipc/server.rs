// Server 是一对多，而且同步的，主要用于 request data 的场景

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::sync::Arc;

use super::id::{coid_alloc, rcvid_alloc, CoidHandler, RcvidHandler};

pub struct Server {
    conn: Vec<CoidHandler>,
    request: Vec<Arc<Msg>>,
    response: BTreeMap<usize, Arc<Msg>>,
}

impl Server {
    pub fn new() -> Self {
        Self { conn: Vec::new(), request: Vec::new(), response: BTreeMap::new() }
    }

    pub fn connect(&mut self) -> Option<usize> {
        let coid = coid_alloc()?;
        let id = coid.0;
        self.conn.push(coid);        
        Some(id)
    }

    pub fn send_request(&mut self, coid: usize, data: Arc<Vec<u8>>) -> Option<usize> {
        let msg: Arc<Msg> = Arc::new(Msg::new(coid, data)?);
        let rcvid = msg.rcvid.0;
        self.request.push(msg);
        Some(rcvid)
    }

    pub fn recv_request(&mut self) -> Option<Arc<Msg>> {
        // 从顶端弹出一个 msg
        self.request.pop()
    }

    pub fn recv_response(&mut self, rcvid: usize) -> Option<Arc<Msg>> {
        self.response.remove(&rcvid)
    }

    pub fn send_response(&mut self, rcvid: usize, data: Arc<Vec<u8>>) {
        let msg = Arc::new(Msg::new_with_rcvid(RcvidHandler::new_with_rcvid(rcvid), data));
        self.response.insert(rcvid, msg);
    }
}

pub struct Msg {
    rcvid: RcvidHandler,
    _coid: usize,
    data: Arc<Vec<u8>>,
}

impl Msg {
    pub fn new(coid: usize, data: Arc<Vec<u8>>) -> Option<Self> {
        let rcvid = rcvid_alloc()?;
        Some(Self { rcvid, _coid: coid, data })
    }

    pub fn new_with_rcvid(rcvid: RcvidHandler, data: Arc<Vec<u8>>) -> Self {
        Self { rcvid, _coid: 0, data }
    }

    pub fn rcvid(&self) -> usize {
        self.rcvid.0
    }

    pub fn data(&self) -> Arc<Vec<u8>> {
        self.data.clone()
    }
}