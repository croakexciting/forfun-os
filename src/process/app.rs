use core::borrow::BorrowMut;
use core::cell::RefMut;
use core::arch::asm;

use crate::mm::allocator::ASID_ALLOCATOR;
use crate::mm::MemoryManager;
use crate::process::switch::__switch;
use crate::config::*;
use crate::trap::context::TrapContext;
use crate::utils::timer::nanoseconds;
use crate::utils::type_extern::RefCellWrap;

use alloc::vec;
use alloc::vec::Vec;
use riscv::register::satp;

use super::context::SwitchContext;

pub struct AppManagerInner {
    started: bool,
    current: usize,
    apps: Vec<Process>,
    idle_ctx: SwitchContext,
}

impl AppManagerInner {
    pub fn new() -> Self {
        AppManagerInner {
            started: false,
            current: 0,
            apps: vec![],
            // idle process is a unstop loop process
            idle_ctx: SwitchContext::new(0, 0),
        }
    }

    // pub fn app(&self)
    fn app(&mut self, id: usize) -> &mut Process {
        self.apps[id].borrow_mut()
    }

    // get idle ctx
    pub fn idle_ctx(&mut self) -> *mut SwitchContext {
        &mut self.idle_ctx as *mut _
    }

    // return app id, if create failed, return -1
    pub fn create_app(&mut self, tick: usize, elf_data: &[u8]) -> i32 {
        // just add a process at the tail
        let app_id = self.apps.len();
        if app_id < MAX_APP_NUM {
            let mut process = Process::new(tick);
            // load elf
            let r = process.load_elf(elf_data);
            if let Err(e) = r {
                println!("load elf error: {}", e);
                return -2;
            }
            println!("load elf success");
            self.apps.push(process);
            app_id as i32
        } else {
            println!("The app pool now is full, can't add new app");
            return -1;
        }
    }

    pub fn activate_app(&mut self, id: usize) {
        self.apps[id].activate();
    } 

    fn current_app(&mut self) -> &mut Process {
        self.app(self.current)
    }

    fn next_app(&mut self) -> &mut Process {
        assert!(self.apps.len() > 0, "The app vector is empty!!!");
        if self.started {
            // When the next api be called, there must be at least one apps in vector
            let next = (self.current + 1) % self.apps.len();
            self.current = next;
            self.app(next)
        } else {
            self.started = true;
            self.app(0)
        }
    }
}

pub struct Process {
    pub tick: usize,
    pub status: ProcessStatus,
    
    ctx: SwitchContext,
    mm: MemoryManager,
    asid:u16
}

impl Process {
    // new 只会创建一个完全空白，无法运行的进程，需要 load_elf 才可使用
    pub fn new(tick: usize) -> Self {
        Process {
            tick,
            status: ProcessStatus::UNINIT,
            ctx: SwitchContext::bare(),
            mm: MemoryManager::new(),
            asid: ASID_ALLOCATOR.exclusive_access().alloc().unwrap(),
        }
    }

    pub fn set_status(&mut self, status: ProcessStatus) {
        self.status = status;
    }

    pub fn ctx_ptr(&mut self) -> *mut SwitchContext {
        self.ctx.borrow_mut() as *mut _
    }
    
    fn satp(&mut self) -> usize {
        8usize << 60 | (self.asid as usize) << 44 | self.mm.root_ppn().0
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
}

#[derive(Copy, Clone, Debug)]
pub enum ProcessStatus {
    UNINIT,
    READY,
    // running status with tick number
    RUNNING(usize),
    // sleep status with start and duration timestamp(ns) 
    SLEEP(usize, usize),
    EXITED,
}

// 简化版的任务控制块，如果考虑后期加入线程的话，似乎 mm 不应该放在这
pub struct AppManager {
    inner: RefCellWrap<AppManagerInner>,
}

impl AppManager {
    pub unsafe fn new() -> Self {
        Self {
            inner: RefCellWrap::new(AppManagerInner::new())
        }
    }

    pub fn inner_access(&self) -> RefMut<'_, AppManagerInner> {
        self.inner.exclusive_access()
    }

    pub fn run_apps(&self) -> ! {
        use ProcessStatus::*;
        loop {
            let mut inner = self.inner_access();
            let idle_ctx = inner.idle_ctx();

            // 更新当前任务时间片，如果时间片还有剩余，切换回当前任务
            let current = inner.current_app();
            if let RUNNING(tick) = current.status {
                if tick - 1 > 0 {
                    let current_ctx_ptr = current.ctx_ptr();
                    current.set_status(RUNNING(tick - 1));
                    drop(inner);
                    unsafe {__switch(idle_ctx, current_ctx_ptr);}
                    continue;
                }
            }

            let next = inner.next_app();
            let next_ctx_ptr = next.ctx_ptr();
            match next.status {
                READY => unsafe {
                    next.set_status(RUNNING(next.tick));
                    next.activate();
                    // TODO: 需要考虑下这个地方，因为切换页表后，执行 __switch 似乎有点问题，但是 kernel 使用 identical 模式，似乎又是没问题的
                    drop(inner);
                    __switch(idle_ctx, next_ctx_ptr);
                },
                RUNNING(_) => unsafe {
                    next.set_status(SLEEP(nanoseconds(), 0));
                    next.activate();
                    drop(inner);
                    __switch(idle_ctx, next_ctx_ptr);
                }
                SLEEP(a, b) => unsafe {
                    if a + b < nanoseconds() {
                        next.set_status(RUNNING(next.tick));
                        next.activate();
                        drop(inner);
                        __switch(idle_ctx, next_ctx_ptr);
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
        let current_ctx_ptr = inner.current_app().ctx_ptr();
        drop(inner);
        unsafe {
            __switch(current_ctx_ptr, idle_ctx);   
        }
    }

    pub fn sleep(&self, duration: usize) {
        let mut inner = self.inner_access();
        let idle_ctx = inner.idle_ctx();
        let current_app = inner.current_app();
        
        // set current time and sleep time
        current_app.set_status(ProcessStatus::SLEEP(nanoseconds(), duration));
        let current_ctx_ptr: *mut SwitchContext = current_app.ctx_ptr();
        drop(inner);
        unsafe {
            __switch(current_ctx_ptr, idle_ctx);
        }
    }

    pub fn exit(&self, _exit_code: i32) -> ! {
        let mut inner = self.inner_access();
        let idle_ctx = inner.idle_ctx();
        let current_ctx_ptr = inner.current_app().ctx_ptr();
        inner.current_app().set_status(ProcessStatus::EXITED);
        // println!("[kernel] Application exited with code {}", _exit_code);
        drop(inner);
        unsafe {
            __switch(current_ctx_ptr, idle_ctx);
            unreachable!()
        }
    }

    pub fn create_app(&self, tick: usize, elf: &[u8]) -> i32 {
        let mut inner = self.inner_access();
        inner.create_app(tick, elf)
    }

    pub fn activate_app(&self, id: usize) {
        let mut inner = self.inner_access();
        inner.activate_app(id)
    }
}