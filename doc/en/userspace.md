## 1 Introduction

This page will introduce how to develop userspace program. You can develop a simple program based on a simple library.

In user/src, you can find it's actually a rust library crate, this is the Forfun OS standard library. Libs provide all syscalls and main function caller.

Now the userspace program source code locate at user/src/bin directory, I will give a independent project example later.

## 2 Main function

For most software engineer, main function is the beginning of everything. But before main function is called, we still have some work to-do. 

The _start function is put in the .text.entry section and defined as the entry in linker scripts. In the _start function, we request 512KB space for heap and initialize heap allocator, then we call main() function in exit syscall to ensure the process will exit.

```
#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    // 512KB, 继续增加一个动态内存分配功能
    init_heap(4096 * 128);
    exit(main());
    panic!("unreachable after sys_exit!");
}
```

> In Linux, windows or other modern OS, works before main function is pretty complex. But the principle is the same, just prepare the runtime env for program.

## 3 Syscall

Kernel handle syscall by syscall id, you can find them in user/src/syscall.rs

```
const SYSCALL_READ: usize = 0;
const SYSCALL_WRITE: usize = 1;
const SYSCALL_OPEN: usize = 2;
const SYSCALL_CLOSE: usize = 3;
const SYSCALL_LSEEK: usize = 4;
const SYSCALL_SIZE: usize = 5;
const SYSCALL_MMAP: usize = 9;
const SYSCALL_UMMAP: usize = 10;
const SYSCALL_MMAP_WITH_ADDR: usize = 11;
const SYSCALL_SIG: usize = 12;
const SYSCALL_SIGACTION: usize = 13;
const SYSCALL_SIGPROCMASK: usize = 14;
const SYSCALL_SIGRETURN: usize = 15;
const SYSCALL_PIPE: usize = 22;
const SYSCALL_YIELD: usize = 24;
const SYSCALL_NANOSLEEP: usize = 35;
const SYSCALL_GETPID: usize = 39;
const SYSCALL_FORK: usize = 57;
const SYSCALL_EXEC: usize = 59;
const SYSCALL_EXIT: usize = 60;
const SYSCALL_WAIT: usize = 61;
const SYSCALL_KILL: usize = 62;
const SYSCALL_SHM_OPEN: usize = 70;
const SYSCALL_SEM_OPEN: usize = 80;
const SYSCALL_SEM_WAIT: usize = 81;
const SYSCALL_SEM_RAISE: usize = 82;
const SYSCALL_SRV_CREATE: usize = 90;
const SYSCALL_SRV_CONNECT: usize = 91;
const SYSCALL_SRV_REQUEST: usize = 92;
const SYSCALL_SRV_RECV: usize = 93;
const SYSCALL_SRV_REPLY: usize = 94;
```

Actually, I want to use the same syscall ID as linux at first, but It's not easy, so I give up and define ID randomly.

Also, the lib provide several syscall functions for app. For example

```
pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buffer.as_ptr() as usize, buffer.len(), 0])
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len(), 0])
}
```

These functions are the warpper of **syscall**, let's take a look at the syscall function. Just like we have mentioned before, set syscall id and params into registers, then run ecall (riscv) or svc (aarch64).

```
fn syscall(id: usize, args: [usize; 4]) -> isize {
    let mut ret: isize;
    #[cfg(feature = "riscv64")]
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x13") args[3],
            in("x17") id
        );
    }

    #[cfg(feature = "aarch64")]
    unsafe {
        asm!(
            "svc #0",
            inlateout("x0") args[0] => ret,
            in("x1") args[1],
            in("x2") args[2],
            in("x3") args[3],
            in("x8") id
        );
    }

    ret
}
```

## 4 Example

Finally, we can develop some simple userspace program. For example, the fork test, fork a child process and wait. Very simple, show as below

```
# user/src/bin/fork_test.rs
fn main() -> i32 {
    println!("fork syscall test");
    let pid = sys_fork();
    if pid == 0 {
        println!("child process");
    } else if pid > 0 {
        println!("parent process");
        loop {
            if sys_wait(pid as usize) < 0 {
                sys_yield();
            } else {
                println!("child process done");
                break;
            }
        }
    } else {
        println!("fork failed");
    }
    0
}
```

Now, the most complex user app is [shell](../../user/src/bin/shell.rs), our toy kernel is too simple to support complex userspace app.

## 5 Package

After we develop and build the userspace apps, we package them into a filesystem and flash the filesystem into block device (In qemu, we can directly load the filesystem bin file).

Kernel will load the filesystem and find apps by name. Kernel will load and run shell by default, you can type app name in shell command line and run it.

You can use **make createfs** to package apps into filesystem.

## 6 Conclusion

Because the Forfun OS is pretty simple, I develop some userspace program just for kernel features test. 

Maybe later, after add more kernel features, I will migrate some busybox apps to my OS.