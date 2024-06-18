use core::borrow::BorrowMut;
use core::cell::RefMut;
use core::arch::asm;

use crate::mm::allocator::{asid_alloc, AisdHandler};
use crate::mm::basic::VirtPage;
use crate::mm::MemoryManager;
use crate::process::switch::__switch;
use crate::trap::context::TrapContext;
use crate::utils::timer::nanoseconds;
use crate::utils::type_extern::RefCellWrap;

use alloc::sync::{Arc, Weak};
use spin::mutex::Mutex;
use alloc::{format, vec};
use alloc::vec::Vec;
use riscv::register::satp;

use super::context::SwitchContext;
use super::pid::{self, PidHandler};

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
            let idle_ctx = inner.idle_ctx();
            // 暂时简化处理，如果获取不到当前任务，会直接 panic
            let current = inner.current_task().unwrap();
            let current_status = current.lock().status;
            if let RUNNING(tick) = current_status {
                if tick - 1 > 0 {
                    let current_ctx_ptr = current.lock().ctx_ptr();
                    current.lock().set_status(RUNNING(tick - 1));
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
                    drop(inner);

                    unsafe { __switch(idle_ctx, next_ctx_ptr); }
                },
                RUNNING(_) => unsafe {
                    next.lock().set_status(SLEEP(nanoseconds(), 0));
                    next.lock().activate();
                    drop(inner);
                    unsafe { __switch(idle_ctx, next_ctx_ptr); }
                }
                SLEEP(a, b) => unsafe {
                    if a + b < nanoseconds() {
                        let tick = next.lock().tick;
                        next.lock().set_status(RUNNING(tick));
                        next.lock().activate();
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
        let current_ctx_ptr = inner.current_task().unwrap().lock().ctx_ptr();
        drop(inner);
        unsafe { __switch(current_ctx_ptr, idle_ctx); }
    }

    pub fn sleep(&self, duration: usize) {
        let mut inner = self.inner_access();
        let current = inner.current_task().unwrap();
        current.lock().set_status(ProcessStatus::SLEEP(nanoseconds(), duration));
        drop(inner);

        self.back_to_idle();
    }

    pub fn exit(&self, exit_code: isize) -> ! {
        let mut inner = self.inner_access();
        let current = inner.current_task().unwrap();
        let idle_ctx = inner.idle_ctx();
        let current_ctx_ptr = current.lock().ctx_ptr();
        current.lock().set_status(ProcessStatus::EXITED(exit_code));
        let pid = current.lock().pid.clone();
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

    pub fn create_initproc(&self, tick: usize, elf: &[u8]) -> isize {
        let mut inner = self.inner_access();
        inner.create_initproc(tick, elf)
    }

    pub fn remap(&self, vpn: VirtPage) -> Result<(), &'static str> {
        let mut inner = self.inner_access();
        inner.remap(vpn)
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
}

impl AppManagerInner {
    pub fn new() -> Self {
        AppManagerInner {
            started: false,
            current: 0,
            initproc: None,
            tasks: vec![],
            // idle process is a unstop loop process
            idle_ctx: SwitchContext::new(0, 0),
        }
    }

    // pub fn app(&self)
    fn task(&mut self, id: usize) -> Result<Arc<Mutex<Process>>, &'static str> {
        if let Some(task) = self.tasks[id].upgrade() {
            Ok(task)
        } else {
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

    fn current_task(&mut self) -> Result<Arc<Mutex<Process>>, &'static str> {
        self.task(self.current)
    }

    fn next_task(&mut self) -> Result<Arc<Mutex<Process>>, &'static str> {
        assert!(self.tasks.len() > 0, "The app vector is empty!!!");
        if self.started {
            // When the next api be called, there must be at least one apps in vector
            let next = (self.current + 1) % self.tasks.len();
            self.current = next;
            self.task(next)
        } else {
            self.started = true;
            self.task(0)
        }
    }

    pub fn fork(&mut self) -> isize {
        let child = self.current_task().unwrap().lock().fork();
        let pid = child.upgrade().unwrap().lock().pid.0;
        self.tasks.push(child);
        pid as isize
    }

    pub fn remap(&mut self, vpn: VirtPage) -> Result<(), &'static str> {
        self.current_task().unwrap().lock().remap(vpn)
    }
}

pub struct Process {
    pub tick: usize,
    pub status: ProcessStatus,
    pub pid: PidHandler,
    pub parent: Option<usize>,
    pub children: Vec<Arc<Mutex<Self>>>,
    
    ctx: SwitchContext,
    mm: MemoryManager,
    asid: AisdHandler,
}

impl Process {
    // new 只会创建一个完全空白，无法运行的进程，需要 load_elf 才可使用
    pub fn new(tick: usize) -> Self {
        Process {
            tick,
            status: ProcessStatus::UNINIT,
            pid: pid::alloc().unwrap(),
            parent: None,
            children: Vec::new(),
            ctx: SwitchContext::bare(),
            mm: MemoryManager::new(),
            asid: asid_alloc().unwrap(),
        }
    }

    pub fn fork(&mut self) -> Weak<Mutex<Self>> {
        let mut mm = MemoryManager::new();
        mm.fork(&mut self.mm);
        let child = Arc::new(Mutex::new(
            Self {
                tick: self.tick,
                status: ProcessStatus::READY,
                pid: pid::alloc().unwrap(),
                parent: Some(self.pid.0),
                children: Vec::new(),
                ctx: self.ctx,
                mm,
                asid: asid_alloc().unwrap(),
            }
        ));

        let weak = Arc::downgrade(&child);
        self.children.push(child);
        weak
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
        let (sp, pc) = self.mm.load_elf(data)?;

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

    // 出现页错误时，重新 map
    pub fn remap(&mut self, vpn: VirtPage) -> Result<(), &'static str> {
        self.mm.remap(vpn)
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
    EXITED(isize),
}
