## 1 Introduction

This page will introduces the design and implementation details of the task schedule function

The [rcore task switch doc](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/index.html) explains the trap mechanism very clear, you can get more basic knowledge from the doc.

> Task schedule: The kernel executes tasks in turns, making each task believe it has exclusive control of the CPU

Task schedule includes two parts:

- Select next running task
- Task switch

## 2 Task switch process

Task switch process show as below

<img src="../../drawio/switch_en.svg" name="Task switch process" >

### 2.1 Task Context

Kernel switch task flow by switch task context. Store current task context from registers and put next task context into registers.

```
pub struct SwitchContext {
    /*
        Actually, task context only save callee-saved registers and ra

        Since the context switch of a certain task always happens when executing __switch, the ra register point to the next instruction of __switch, and saved into task context

        When switch back to this task, the callee-saved register is restored, after execute ret, jump back to the function caller and cpu will automatically continue execution from where the task was interrupted

        Therefore, for a certain task, it will only perceive that it is executing a function normally. Even though it was interrupted in the middle, it has no knowledge of the interruption

        That's why only save callee-saved registers in task context
    */

    ra: usize,
    sp: usize,
    s: [usize; 12],
}
```

### 2.2 Kernel Stack

Kernel stack is very confused. Actually, kernel stack should be divided into two categories. 

- The first category is the stack the kernel, as software itself, must include. The stack is exactly the same as the stack found in application software. This stack only used at kernel startup stage.
- The second category is what we commonly refer to as the kernel stack. Every task need a individual kernel stack area, When user space task enter kernel, use it's own kernel stack, trap context will saved in this stack.

**Therefore, if you want to switch to a certain task, you first need to switch to its corresponding kernel stack. Then, by executing the trap recovery process, it will naturally switch to that task. You can find that task context include sp register**

**You can imagine that kernel is like a power source, and it has many plugs (kernel stacks), but it can only power one plug at a time. Applications are the devices that need power, and each app is plugged into one of the plugs. The power source continuously chooses which plug to switch to next. Once it directs power to a certain plug, the electricity will naturally flow to the corresponding device.**

### 2.3 Control flow switch

Each task has a task flow, besides there is an idle flow. All task run on their own task flow, scheduler run on idle flow.

We use __switch assembly code switch Task Context. As we mentioned above, the task context store callee-saved registers. So the context can reprensent a function, and function run on control flow. so we can get

**switch task context = switch function = switch control flow = switch running task**

This is why siwtching task context can switch task

<img src="../../drawio/switch_detail_en.svg" name="switch detail" >

**In this process, every task believes it has only execute a simple trap function. But the kernel stop the trap process, and switch to another task. Task complete the second half trap process until kernel switch to it again.**

### 2.4 Task Control Block

Every task(process or thread) has a task control block, which store task's informations. In forfun os, because of we only support process now, so I use Process to name TCB, Process struct show as below

```
pub struct Process {
    pub tick: usize,
    pub status: ProcessStatus,
    pub pid: PidHandler,
    pub parent: Option<usize>,
    pub children: BTreeMap<usize, Arc<Mutex<Self>>>,
    
    ctx: SwitchContext,
    mm: MemoryManager,
    asid: AisdHandler,
    fds: Vec<Option<Arc<dyn File>>>,
    signals: SignalFlags,
    signals_mask: SignalFlags,
    signal_actions: Vec<Option<SignalAction>>,
    trap_ctx_backup: Option<TrapContext>,
}
```

The most important fields are switch context, memory manager

## 3 Scheduler

Scheduler select next running task, and executed in run_task function which run on idle control flow.

Now I use the most simple schedule strategy, Round Robin.

```
# next_task() function is the scheduler

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
            Err(e) => {
                // println!("[kernel] get task {} failed, {}", self.current, e);
                self.next_task()
            } 
        }
    } else {
        self.started = true;
        // Whatever，pid 0 process must exists
        Some(self.task(0).unwrap())
    }
}
```

## 4 Conclusion

This chapter introduce the task switch process. In physical mode, it's not easy to separate each task's memory area. Next chapter, I will introduce how to convert to virtual mode.