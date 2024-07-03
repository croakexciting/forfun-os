use core::borrow::BorrowMut;
use core::cell::RefMut;
use core::arch::asm;
use core::ops::{BitAnd, BitOr};

use crate::driver::block::qemu_blk::{self, QemuBlk};
use crate::driver::block::BlockDevice;
use crate::ipc::id::RcvidHandler;
use crate::ipc::server::{Msg, Server};
use crate::ipc::pipe::Pipe;
use crate::file::stdio::Stdout;
use crate::file::File;
use crate::ipc::semaphore::Semaphore;
use crate::ipc::shm::Shm;
use crate::mm::allocator::{asid_alloc, AisdHandler};
use crate::mm::area::UserBuffer;
use crate::mm::basic::{PhysAddr, VirtAddr, VirtPage, PAGE_SIZE};
use crate::mm::MemoryManager;
use crate::process::switch::__switch;
use crate::trap::context::TrapContext;
use crate::utils::timer::nanoseconds;
use crate::utils::type_extern::RefCellWrap;

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::collections::BTreeMap;
use bitflags::Flags;
use spin::mutex::Mutex;
use alloc::{format, vec};
use alloc::vec::Vec;
use riscv::register::satp;

use super::context::SwitchContext;
use super::pid::{self, PidHandler};
use crate::ipc::signal::{self, SignalAction, SignalFlags, SIG_NUM};

pub struct TaskManager {
    inner: RefCellWrap<AppManagerInner>,
}

impl TaskManager {
    pub unsafe fn new() -> Self {
        Self {
            inner: RefCellWrap::new(AppManagerInner::new())
        }
    }

    pub fn inner_access(&self) -> RefMut<'_, AppManagerInner> {
        self.inner.exclusive_access()
    }

    pub fn run_task(&self) -> ! {
        use ProcessStatus::*;
        loop {
            let mut inner = self.inner_access();
            // check semaphore
            inner.check_sem();

            let idle_ctx = inner.idle_ctx();
            // 暂时简化处理，如果获取不到当前任务，会直接 panic
            let current = inner.current_task(false).unwrap();
            let current_status = current.lock().status;
            if let RUNNING(tick) = current_status {
                if tick - 1 > 0 {
                    let current_ctx_ptr = current.lock().ctx_ptr();
                    current.lock().set_status(RUNNING(tick - 1));
                    drop(current);
                    drop(inner);
                    unsafe {__switch(idle_ctx, current_ctx_ptr);}
                    continue;
                }
            }

            let next = inner.next_task().unwrap();
            let next_ctx_ptr = next.lock().ctx_ptr();
            let next_status = next.lock().status;
            match next_status {
                READY => unsafe {
                    let tick = next.lock().tick;
                    next.lock().set_status(RUNNING(tick));
                    next.lock().activate();
                    // TODO: 需要考虑下这个地方，因为切换页表后，执行 __switch 似乎有点问题，但是 kernel 使用 identical 模式，似乎又是没问题的
                    drop(next);
                    drop(inner);

                    unsafe { __switch(idle_ctx, next_ctx_ptr); }
                },
                RUNNING(_) => unsafe {
                    next.lock().set_status(SLEEP(nanoseconds(), 0));
                    next.lock().activate();
                    drop(next);
                    drop(inner);
                    unsafe { __switch(idle_ctx, next_ctx_ptr); }
                }
                SLEEP(a, b) => unsafe {
                    if a + b < nanoseconds() {
                        let tick = next.lock().tick;
                        next.lock().set_status(RUNNING(tick));
                        next.lock().activate();
                        drop(next);
                        drop(inner);
                        unsafe { __switch(idle_ctx, next_ctx_ptr); }
                    } else {
                        continue;
                    }
                }
                _ => {
                    continue;
                },
            }
        }
    }

    pub fn back_to_idle(&self) {
        let mut inner = self.inner_access();
        let idle_ctx = inner.idle_ctx();
        let current_ctx_ptr = inner.current_task(false).unwrap().lock().ctx_ptr();
        drop(inner);
        unsafe { __switch(current_ctx_ptr, idle_ctx); }
    }

    pub fn sleep(&self, duration: usize) {
        let mut inner = self.inner_access();
        let current = inner.current_task(false).unwrap();
        current.lock().set_status(ProcessStatus::SLEEP(nanoseconds(), duration));
        drop(current);
        drop(inner);

        self.back_to_idle();
    }

    pub fn exit(&self, exit_code: isize) -> ! {
        let mut inner = self.inner_access();
        let current = inner.current_task(false).unwrap();
        let idle_ctx = inner.idle_ctx();
        let current_ctx_ptr = current.lock().ctx_ptr();
        current.lock().set_status(ProcessStatus::EXITED(exit_code));
        let pid = current.lock().pid.clone();
        drop(current);
        drop(inner);
        unsafe {
            __switch(current_ctx_ptr, idle_ctx);
            unreachable!()
        }
    }

    pub fn fork(&self) -> isize {
        let mut inner = self.inner_access();
        inner.fork()
    }

    pub fn exec(&self, elf: usize) -> isize {
        let mut inner = self.inner_access();
        inner.exec(elf)
    }

    pub fn create_initproc(&self, tick: usize, elf: &[u8]) -> isize {
        let mut inner = self.inner_access();
        inner.create_initproc(tick, elf)
    }

    pub fn cow(&self, vpn: VirtPage) -> Result<(), &'static str> {
        let mut inner = self.inner_access();
        inner.cow(vpn)
    }

    pub fn wait(&self, pid: isize) -> isize {
        let mut inner = self.inner_access();
        inner.wait(pid)
    }

    pub fn write(&self, fd: usize, buf: *mut u8, len: usize) -> isize {
        let mut inner = self.inner_access();
        inner.write(fd, buf, len)
    }

    pub fn create_pipe(&self, size: usize) -> (usize, usize) {
        let mut inner = self.inner_access();
        inner.create_pipe(size)
    }

    pub fn read(&self, fd: usize, buf: *mut u8, len: usize) -> isize {
        let mut read = 0;
        loop {
            let mut inner = self.inner_access();
            let ret = inner.read(fd, buf, len);
            if ret < 0 {
                return ret;
            }

            read += ret;
            if (read as usize) < len {
                drop(inner);
                self.back_to_idle();
            } else {
                return read
            }
        }
    }

    pub fn sigaction(&self, signal: usize, handler: usize) -> isize {
        let mut inner = self.inner_access();
        inner.sigaction(signal, handler)
    }

    pub fn set_signal(&self, pid: Option<usize>, signal: usize) -> isize {
        let mut inner = self.inner_access();
        inner.set_signal(pid, signal)
    }

    pub fn set_signalmask(&self, signal: usize) -> isize {
        let mut inner = self.inner_access();
        inner.set_signalmask(signal)
    }

    pub fn signal_handler(&self) -> SignalCode {
        let mut inner = self.inner.exclusive_access();
        inner.signal_check()
    }

    pub fn save_trap_ctx(&self) {
        let mut inner = self.inner.exclusive_access();
        inner.save_trap_ctx()
    }

    pub fn sigreturn(&self) -> isize {
        let mut inner = self.inner.exclusive_access();
        inner.sigreturn()
    }

    pub fn getpid(&self) -> usize {
        let mut inner = self.inner.exclusive_access();
        inner.getpid()
    }

    pub fn mmap(&self, size: usize, permission: usize) -> isize {
        let mut inner = self.inner.exclusive_access();
        inner.mmap(size, permission)
    }

    pub fn ummap(&self, addr: usize) -> isize {
        let mut inner = self.inner.exclusive_access();
        inner.ummap(addr)
    }

    pub fn mmap_with_addr(&self, pa: usize, size: usize, permission: usize) -> isize {
        let mut inner = self.inner.exclusive_access();
        inner.mmap_with_addr(pa, size, permission)
    }

    pub fn create_or_open_shm(&self, name: String, size: usize, permission: usize) -> isize {
        assert_eq!(size % PAGE_SIZE, 0);
        let mut inner = self.inner.exclusive_access();
        inner.create_or_open_shm(name, size / PAGE_SIZE, permission)
    }

    pub fn open_sem(&self, name: String) -> isize {
        let mut inner = self.inner.exclusive_access();
        inner.open_sem(name)
    }

    pub fn wait_sem(&self, name: String) -> isize {
        let mut inner = self.inner.exclusive_access();
        let r = inner.wait_sem(name);
        drop(inner);

        if r == 0 {
            self.back_to_idle();
        }

        r
    }

    pub fn raise_sem(&self, name: String) -> isize {
        let mut inner = self.inner.exclusive_access();
        inner.raise_sem(name)
    }

    pub fn create_server(&self, name: String) -> isize {
        let mut inner = self.inner.exclusive_access();
        inner.create_server(name)
    }

    // return coid
    pub fn connect_server(&self, name: String) -> isize {
        let mut inner = self.inner.exclusive_access();
        inner.connect_server(name)
    }

    pub fn request(&self, coid: usize, data: Arc<Vec<u8>>) -> Option<Arc<Vec<u8>>> {
        let mut inner = self.inner.exclusive_access();
        let rcvid = inner.send_request(coid, data);
        if rcvid < 0 {
            return None;
        }
        drop(inner);

        loop {
            let mut inner = self.inner.exclusive_access();
            // waiting for response
            if let Some(msg) = inner.recv_response(rcvid as usize) {
                if msg.rcvid() == rcvid as usize {
                    return Some(msg.data())
                }
            } else {
                // back to idle
                drop(inner);
                self.back_to_idle();
            }
        }
    }

    pub fn recv_request(&self, name: String) -> Option<(usize, Arc<Vec<u8>>)> {
        loop {
            let mut inner = self.inner.exclusive_access();
            if let Some(msg) = inner.recv_request(name.to_owned()) {
                return Some((msg.rcvid(), msg.data()));
            } else {
                drop(inner);
                self.back_to_idle();
            }
        }
    }

    pub fn reply_request(&self, rcvid: usize, data: Arc<Vec<u8>>) -> isize {
        let mut inner = self.inner.exclusive_access();
        inner.send_response(rcvid, data)
    }
}

pub struct AppManagerInner {
    started: bool,
    current: usize,
    
    // 这里存储的是 initproc 的实例
    initproc: Option<Arc<Mutex<Process>>>,
    // 存储 process 的 weak pointer, 用于调度
    tasks: Vec<Weak<Mutex<Process>>>,
    idle_ctx: SwitchContext,
    // name -> shm
    // 目前简单考虑，命名 ipc 的 key 都使用数字，后面考虑支持字符串
    named_shm: BTreeMap<String, Shm>,
    named_sem: BTreeMap<String, Arc<Mutex<Semaphore>>>,
    named_srv: BTreeMap<String, Arc<Mutex<Server>>>,
    srv_conn: BTreeMap<usize, Weak<Mutex<Server>>>,
    session: BTreeMap<usize, Weak<Mutex<Server>>>,
}

impl AppManagerInner {
    pub fn new() -> Self {
        AppManagerInner {
            started: false,
            current: 0,
            initproc: None,
            tasks: Vec::new(),
            // idle process is a unstop loop process
            idle_ctx: SwitchContext::new(0, 0),
            named_shm: BTreeMap::new(),
            named_sem: BTreeMap::new(),
            named_srv: BTreeMap::new(),
            srv_conn: BTreeMap::new(),
            session: BTreeMap::new(),
        }
    }

    // pub fn app(&self)
    fn task(&mut self, id: usize) -> Result<Arc<Mutex<Process>>, &'static str> {
        if let Some(task) = self.tasks[id].upgrade() {
            Ok(task)
        } else {
            self.tasks.remove(id);
            Err("task instance not exists now.")
        }
    }


    // get idle ctx
    pub fn idle_ctx(&mut self) -> *mut SwitchContext {
        &mut self.idle_ctx as *mut _
    }

    // return app id, if create failed, return -1
    // only initproc is created, other's created by fork
    pub fn create_initproc(&mut self, tick: usize, elf_data: &[u8]) -> isize {
        // just add a process at the tail
        let mut initproc = Process::new(tick);
        // load elf
        let r = initproc.load_elf(elf_data);
        if let Err(e) = r {
            println!("[kernel] initproc load elf error: {}", e);
            return -1;
        }
        println!("[kernel] initproc load elf success");
        let initproc_arc = Arc::new(Mutex::new(initproc));
        self.tasks.push(Arc::downgrade(&initproc_arc));
        self.initproc = Some(initproc_arc);
        0
    }

    pub fn activate_task(&mut self, id: usize) -> Result<(), &'static str> {
        if let Some(task) = self.tasks[id].upgrade() {
            Ok(())
        } else {
            Err("task instance not exists now.")
        }
    } 

    fn current_task(&mut self, must: bool) -> Option<Arc<Mutex<Process>>> {
        match self.task(self.current) {
            Ok(p) => {
                Some(p)
            }
            Err(_) => {
                if must == false {
                    return self.next_task()
                } else {
                    return None;
                }
            } 
        }
    }

    fn next_task(&mut self) -> Option<Arc<Mutex<Process>>> {
        assert!(self.tasks.len() > 0, "The app vector is empty!!!");
        if self.started {
            // When the next api be called, there must be at least one apps in vector
            let next = (self.current + 1) % self.tasks.len();
            self.current = next;
            match self.task(self.current) {
                Ok(p) => {
                    Some(p)
                }
                Err(_) => {
                    self.next_task()
                } 
            }
        } else {
            self.started = true;
            // 无论如何，至少有一个初始进程，pid 为 0 的进程必须存在
            Some(self.task(0).unwrap())
        }
    }

    pub fn fork(&mut self) -> isize {
        let child = Process::fork(self.current_task(true).unwrap().clone());
        let pid = child.upgrade().unwrap().lock().pid.0;
        self.tasks.push(child);
        pid as isize
    }

    pub fn exec(&mut self, elf: usize) -> isize {
        match self.current_task(true).unwrap().lock().exec(elf) {
            Ok(_) => {return 0;}
            Err(e) => {
                println!("[kernel] exec failed {}", e);
                return  -1;
            }
        }
    }

    pub fn cow(&mut self, vpn: VirtPage) -> Result<(), &'static str> {
        self.current_task(true).unwrap().lock().cow(vpn)
    }

    pub fn wait(&mut self, pid: isize) -> isize {
        self.current_task(true).unwrap().lock().wait(pid)
    }

    pub fn write(&mut self, fd: usize, buf: *mut u8, len: usize) -> isize {
        self.current_task(true).unwrap().lock().write(fd, buf, len)
    }

    pub fn create_pipe(&mut self, size: usize) -> (usize, usize) {
        self.current_task(true).unwrap().lock().create_pipe(size)
    }

    pub fn read(&mut self, fd: usize, buf: *mut u8, len: usize) -> isize {
        self.current_task(true).unwrap().lock().read(fd, buf, len)
    }

    pub fn sigaction(&mut self, signal: usize, handler: usize) -> isize {
        self.current_task(true).unwrap().lock().sigaction(signal, SignalAction::new(signal, handler))
    }

    pub fn set_signal(&mut self, pid: Option<usize>, signal: usize) -> isize {
        if let Some(pid) = pid {
            for task in (&self.tasks).into_iter() {
                if let Some(t) = task.upgrade() {
                    if t.lock().pid.0 == pid {
                        return t.lock().set_signal(signal)
                    }
                }
            }
        } else {
            return self.current_task(true).unwrap().lock().set_signal(signal);
        }

        -1
    }

    pub fn set_signalmask(&mut self, mask: usize) -> isize {
        self.current_task(true).unwrap().lock().set_signalmask(mask)
    }

    pub fn signal_check(&mut self) -> SignalCode {
        self.current_task(true).unwrap().lock().signal_check()
    }

    pub fn save_trap_ctx(&mut self) {
        self.current_task(true).unwrap().lock().save_trap_ctx()
    }

    pub fn sigreturn(&mut self) -> isize {
        self.current_task(true).unwrap().lock().sigreturn()
    }

    pub fn getpid(&mut self) -> usize {
        self.current_task(true).unwrap().lock().pid.0
    }

    pub fn mmap(&mut self, size: usize, permission: usize) -> isize {
        self.current_task(true).unwrap().lock().mmap(size, permission)
    }

    pub fn ummap(&mut self, addr: usize) -> isize {
        self.current_task(true).unwrap().lock().ummap(addr.into())
    }

    pub fn mmap_with_addr(&mut self, pa: usize, size: usize, permission: usize) -> isize {
        self.current_task(true).unwrap().lock().mmap_with_addr(pa.into(), size, permission)
    }

    pub fn create_or_open_shm(&mut self, name: String, pn: usize, permission: usize) -> isize {
        let current_task = self.current_task(true).unwrap();
        let pid = current_task.lock().pid.0;
        if let Some(shm) = self.named_shm.get_mut(&name) {
            // map with process memory manager
            shm.map(pid, &mut current_task.lock().mm)
        } else {
            // create a shm
            let mut shm = Shm::new(pn, permission);
            let r = shm.map(pid, &mut current_task.lock().mm);
            self.named_shm.insert(name, shm);
            r
        }
    }

    pub fn close_shm(&mut self, addr: usize, name: String) -> isize {
        let current_task = self.current_task(true).unwrap();
        let pid = current_task.lock().pid.0;
        if let Some(shm) = self.named_shm.get_mut(&name) {
            // map with process memory manager
            let start_vpn: VirtPage = VirtAddr::from(addr).into();
            shm.unmap(pid, start_vpn, &mut current_task.lock().mm);
            0
        } else {
            println!("[kernel] This shm is not exist");
            -1
        }
    }

    pub fn remove_shm(&mut self, name: String) -> isize {
        if let Some(shm) = self.named_shm.get_mut(&name) {
            if shm.users.len() > 0 {
                println!("[kernel] Shm {} still in used, can't remove", name.as_str());
                -2
            } else {
                self.named_shm.remove(&name);
                0
            }
        } else {
            println!("[kernel] This shm is not exist");
            -1
        }
    }

    pub fn open_sem(&mut self, name: String) -> isize {
        if let Some(_) = self.named_sem.get(&name) {
            println!("[kernel] semaphore {} already exists", name.as_str());
            return -1;
        } else {
            let mut sem_ptr = Arc::new(Mutex::new(Semaphore::new()));
            self.named_sem.insert(name, sem_ptr);
            0
        }
    }

    pub fn wait_sem(&mut self, name: String) -> isize {
        let current_task = self.current_task(true).unwrap();
        if let Some(sem) = self.named_sem.get_mut(&name) {
            current_task.lock().set_status(ProcessStatus::WAITING);
            sem.lock().wait(Arc::downgrade(&current_task));
            return 0;
        } else {
            println!("[kernel] semaphore {} not exists", name.as_str());
            return -1;
        }
    }

    pub fn raise_sem(&mut self, name: String) -> isize {
        if let Some(sem) = self.named_sem.get_mut(&name) {
            if let Some(proc_weak_ptr) = sem.lock().raise() {
                if let Some(proc_ptr) = proc_weak_ptr.upgrade() {
                    proc_ptr.lock().set_status(ProcessStatus::READY);
                    return 0;
                }
            }
            println!("[kernel] can't get process instance");
            return -2;
        } else {
            println!("[kernel] semaphore {} not exists", name.as_str());
            return -1;
        }
    }

    pub fn check_sem(&mut self) {
        for (k, v) in &mut self.named_sem {
            if let Some(proc_weak_ptr) = v.lock().check() {
                if let Some(proc_ptr) = proc_weak_ptr.upgrade() {
                    proc_ptr.lock().set_status(ProcessStatus::READY);
                }
            }
        }
    }

    // 感觉 server 这个 ipc 的功能，我设计的非常烂
    // 我觉得 coid 和 rcvid 是不是要合并起来
    pub fn create_server(&mut self, name: String) -> isize {
        if let Some(_) = self.named_srv.get(&name) {
            println!("[kernel] server {} already exists", name.as_str());
            return -1;
        } else {
            let proc: Weak<Mutex<Process>> = Arc::downgrade(&self.current_task(true).unwrap());
            let srv: Arc<Mutex<Server>> = Arc::new(Mutex::new(Server::new()));
            self.named_srv.insert(name, srv);
            0
        }
    }

    pub fn connect_server(&mut self, name: String) -> isize {
        if let Some(srv) = self.named_srv.get(&name) {
            if let Some(coid) = srv.lock().connect() {
                self.srv_conn.insert(coid, Arc::downgrade(srv));
                return coid as isize;
            } else {
                return -1;
            }
        } else {
            return -2;
        }
    }

    pub fn send_request(&mut self, coid: usize, data: Arc<Vec<u8>>) -> isize {
        // send
        if let Some(srv_weak) = self.srv_conn.get_mut(&coid) {
            if let Some(srv) = srv_weak.upgrade() {
                if let Some(rcvid) = srv.lock().send_request(coid, data) {
                    self.session.insert(rcvid, Arc::downgrade(&srv));
                    return rcvid as isize;
                }
            }
        }

        -1
    }

    pub fn recv_request(&mut self, name: String) -> Option<Arc<Msg>> {
        if let Some(srv) = self.named_srv.get(&name) {
            let msg = srv.lock().recv_request()?;
            let rcvid = msg.rcvid();
            Some(msg)
        } else {
            None
        }
    }

    pub fn send_response(&mut self, rcvid: usize, data: Arc<Vec<u8>>) -> isize {
        if let Some(srv_weak) = self.session.get(&rcvid) {
            if let Some(srv) = srv_weak.upgrade() {
                srv.lock().send_response(rcvid, data);
                0
            } else {
                -1
            }
        } else {
            -2
        }
    }

    pub fn recv_response(&mut self, rcvid: usize) -> Option<Arc<Msg>> {
        let srv = self.session.get(&rcvid)?.upgrade()?;
        let msg = srv.lock().recv_response(rcvid)?;
        self.session.remove(&rcvid);
        Some(msg)
    }
}

pub struct Process {
    pub tick: usize,
    pub status: ProcessStatus,
    pub pid: PidHandler,
    pub parent: Option<Weak<Mutex<Self>>>,
    pub children: BTreeMap<usize, Arc<Mutex<Self>>>,
    
    ctx: SwitchContext,
    mm: MemoryManager,
    asid: AisdHandler,
    fds: Vec<Option<Arc<dyn File>>>,
    signals: SignalFlags,
    signals_mask: SignalFlags,
    // 没有 handler，默认啥也不做，忽略
    signal_actions: Vec<Option<SignalAction>>,
    trap_ctx_backup: Option<TrapContext>,
}

impl Process {
    // new 只会创建一个完全空白，无法运行的进程，需要 load_elf 才可使用
    pub fn new(tick: usize) -> Self {
        Process {
            tick,
            status: ProcessStatus::UNINIT,
            pid: pid::alloc().unwrap(),
            parent: None,
            children: BTreeMap::new(),
            ctx: SwitchContext::bare(),
            mm: MemoryManager::new(),
            asid: asid_alloc().unwrap(),
            fds: vec![
                // 0 -> stdin
                None,
                // 1 -> stdout
                Some(Arc::new(Stdout)),
                // 2 -> stderr
                None,
            ],
            signals: SignalFlags::empty(),
            signals_mask: SignalFlags::all(),
            signal_actions: vec![None; SIG_NUM],
            trap_ctx_backup: None,
        }
    }

    pub fn fork(parent: Arc<Mutex<Self>>) -> Weak<Mutex<Self>> {
        let mut mm = MemoryManager::new();
        mm.fork(&mut parent.lock().mm);
        let switch_ctx = SwitchContext::new_with_restore_addr_and_sp();
        let pid = pid::alloc().unwrap();
        let key = pid.0;
        let tick = parent.lock().tick;
        let fds = parent.lock().fds.clone();
        let signals =  parent.lock().signals;
        let signals_mask = parent.lock().signals_mask;
        let signal_actions = parent.lock().signal_actions.clone();

        let child = Arc::new(Mutex::new(
            Self {
                tick,
                status: ProcessStatus::READY,
                pid,
                parent: Some(Arc::downgrade(&parent)),
                children: BTreeMap::new(),
                ctx: switch_ctx,
                mm,
                asid: asid_alloc().unwrap(),
                fds,
                signals,
                signals_mask,
                signal_actions,
                trap_ctx_backup: None,
            }
        ));

        let weak = Arc::downgrade(&child);
        parent.lock().children.insert(key, child);
        weak
    }

    pub fn exec(&mut self, data_addr: usize) -> Result<(), &'static str> {
        let elf = unsafe {
            core::slice::from_raw_parts(data_addr as *mut u8, 4096 * 16)
        };

        // unmap all app area, for load elf again
        self.mm.unmap_app();
        let (sp, pc) = self.mm.load_elf(elf, true)?;
        let trap_ctx = TrapContext::new(pc, sp);
        let kernel_sp = self.mm.runtime_push_context(trap_ctx);
        self.ctx = SwitchContext::new_with_restore_addr(kernel_sp);
        self.set_status(ProcessStatus::READY);
        Ok(())
    }

    // 等待子进程结束，如果结束，回收子进程资源
    // TODO: 在 task manager 中已经释放的进程去掉
    pub fn wait(&mut self, pid: isize) -> isize {
        let mut result = -1;

        for (k, v) in self.children.clone().iter().map(|child| child) {
            if pid == -1 || (pid as usize) == v.lock().pid.0 {
                match v.lock().status {
                    ProcessStatus::EXITED(_) => {
                        self.children.remove(k);
                        result = 0;
                    }
                    _ => {
                        // 还没结束
                        result = -2;
                        break;
                    }
                }
            }
        }

        result
    }

    pub fn set_status(&mut self, status: ProcessStatus) {
        self.status = status;
    }

    pub fn ctx_ptr(&mut self) -> *mut SwitchContext {
        self.ctx.borrow_mut() as *mut _
    }
    
    fn satp(&mut self) -> usize {
        8usize << 60 | (self.asid.0 as usize) << 44 | self.mm.root_ppn().0
    }

    pub fn load_elf(&mut self, data: &[u8]) -> Result<(), &'static str> {
        // 解析 elf 文件到 mm 中
        // 请注意，这里的 sp 是用户栈 sp，而不是 app 对应的内核栈的 app
        let (sp, pc) = self.mm.load_elf(data, false)?;

        // 根据获取的 app pc 和 sp 创建 TrapContext
        let trap_ctx = TrapContext::new(pc, sp);

        // 将 TrapContext push 到 kernel stack 中，并且更新 switch context
        let kernel_sp = self.mm.push_context(trap_ctx);
        self.ctx = SwitchContext::new_with_restore_addr(kernel_sp);

        self.set_status(ProcessStatus::READY);
        Ok(())
    }

    pub fn runtime_load_elf(&mut self, data: &[u8]) -> Result<(), &'static str> {
        // 解析 elf 文件到 mm 中
        // 请注意，这里的 sp 是用户栈 sp，而不是 app 对应的内核栈的 app
        let (sp, pc) = self.mm.load_elf(data, true)?;

        // 根据获取的 app pc 和 sp 创建 TrapContext
        let trap_ctx = TrapContext::new(pc, sp);

        // 将 TrapContext push 到 kernel stack 中，并且更新 switch context
        let kernel_sp = self.mm.push_context(trap_ctx);
        self.ctx = SwitchContext::new_with_restore_addr(kernel_sp);

        self.set_status(ProcessStatus::READY);
        Ok(())
    }

    // 使能虚地址模式，并且将该进程的页表写到 satp 中
    pub fn activate(&mut self) {
        let satp: usize = self.satp();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }

    // 出现页错误时，copy on write
    pub fn cow(&mut self, vpn: VirtPage) -> Result<(), &'static str> {
        self.mm.cow(vpn)
    }

    // write
    pub fn write(&self, fd: usize, buf: *mut u8, len: usize) -> isize {
        let user_buf = UserBuffer::new_from_raw(buf, len);
        if let Some(file) = &self.fds[fd] {
            if file.writable() {
                return file.write(user_buf) as isize;
            } else {
                println!("[kernel] {} file is None", fd);
                return -1;
            }
        }

        println!("[kernel] {} file not in fd table", fd);
        return -2;
    }

    pub fn create_pipe(&mut self, size: usize) -> (usize, usize)  {
        let (read_pipe, write_pipe) = Pipe::new(size);
        self.fds.push(Some(read_pipe));
        let read_fd = self.fds.len() - 1;
        self.fds.push(Some(write_pipe));
        let write_fd = self.fds.len() - 1;
        (read_fd, write_fd)
    }

    pub fn read(&self, fd: usize, buf: *mut u8, len: usize) -> isize {
        let user_buf = UserBuffer::new_from_raw(buf, len);
        if let Some(file) = &self.fds[fd] {
            if file.readable() {
                return file.read(user_buf) as isize;
            } else {
                println!("[kernel] {} file is None", fd);
                return -1;
            }
        }

        println!("[kernel] {} file not in fd table", fd);
        return -2;
    }

    // signal
    pub fn sigaction(&mut self, signal: usize, action: SignalAction) -> isize {
        self.signal_actions[signal] = Some(action);

        0
    }

    pub fn set_signal(&mut self, signal: usize) -> isize {
        let signal = SignalFlags::from_bits_truncate(1 << signal);
        self.signals.insert(signal);
        
        0
    }

    pub fn set_signalmask(&mut self, mask: usize) -> isize {
        self.signals_mask = SignalFlags::from_bits_truncate(mask as u32);

        0
    }

    pub fn signal_check(&mut self) -> SignalCode {
        let signals = self.signals.bitand(self.signals_mask);
        // 如果有多个信号，从低到高返回第一个找到的信号量
        if let Some(e) = signals.check_error() {
            println!("[kernel] Process {}: {}",self.pid.0, e.1);
            self.signals.remove(SignalFlags::from_bits_truncate(1 << e.0));
            return SignalCode::KILL(e.0 as isize);
        }

        if let Some(v) = signals.first_valid() {
            if let Some(a) = self.signal_actions[v] {
                self.signals.remove(SignalFlags::from_bits_truncate(1 << v));
                return SignalCode::Action(a);
            }
        }

        SignalCode::IGNORE
    }

    pub fn save_trap_ctx(&mut self) {
        // save current trap context in self memory space
        self.trap_ctx_backup = Some(self.mm.runtime_pull_context());
    }

    pub fn sigreturn(&mut self) -> isize {
        // save current trap context in self memory space
        if let Some(mut ctx) = self.trap_ctx_backup.to_owned() {
            return self.mm.runtime_push_context(ctx) as isize
        }

        -1
    }

    pub fn mmap(&mut self, size: usize, permission: usize) -> isize {
        if let Some(area) = self.mm.mmap(size, permission).unwrap().upgrade() {
            return VirtAddr::from(area.lock().start_vpn).0 as isize
        } else {
            -1
        }
    }

    pub fn ummap(&mut self, addr: VirtAddr) -> isize {
        self.mm.umap_dyn_area(addr.into())
    }

    pub fn mmap_with_addr(&mut self, pa: PhysAddr, size: usize, permission: usize) -> isize {
        self.mm.mmap_with_addr(pa, size, permission)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ProcessStatus {
    UNINIT,
    READY,
    // running status with tick number
    RUNNING(usize),
    // sleep status with start and duration timestamp(ns) 
    SLEEP(usize, usize),
    WAITING,
    EXITED(isize),
}

pub enum SignalCode {
    IGNORE,
    Action(SignalAction),
    KILL(isize),
}
