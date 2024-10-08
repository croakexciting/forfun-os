## 1 前言

前几章我们介绍了内核实现，但由于是微内核，很多功能需要在用户层实现。本章将介绍如何开发 Forfun OS 用户程序，以及文件系统和 Shell 的实现。

## 2 用户程序

简单起见，用户程序也使用 rust 开发。用户程序运行在 Forfun OS 上，而不是 linux 上，因此也需要使用 no_std 模式编程。

在 user/src 路径下，我们可以看到如下文件，相当于一个基础的 Forfun OS 标准库和编译环境。

- lib.rs rust lib 库的主文件，定义了 entry 函数，初始化了堆
- console.rs: 实现了 rust println! 宏，方便开发
- lang_items.rs: 由于使用 no_std 模式开发，需要实现一些必要的接口，如 panic
- linker.ld: 链接脚本，定义 entry 地址
- syscall.rs: 定义了所有系统调用函数
- signal.rs: 定义了 signal 类别，用处不大，后面可删掉

在这个环境中进行应用程序开发，rust 语言的特性基本都可以使用，只是在使用 syscall 的时候需要了解下 syscall 用法。

应用程序暂且都放在 user/src/bin 文件夹下，后面也可以成独立项目，将编译环境作为 lib 引入


## 3 用户程序示例

下面是一个最简单的示例，可以看出，和 std 模式下开发差不多，只是需要加一些定义

```
hello_world.rs

#![no_std]
#![no_main]

#[macro_use]
extern crate ffos_app;

#[no_mangle] // 让编译器不要修改 main 函数符号名，否则 entry 中找不到了
fn main() -> i32 {
    println!("Hello, world!");
    0
}

```

## 4 文件系统服务

Forfun-OS 的文件系统运行在用户层，其他进程通过 IPC 从文件系统读写文件。相比如宏内核，这种设计读写效率很低，好处是内核不关心文件系统如何设计，对于内核来说，文件系统和其他进程没啥区别。

文件系统的一些概念介绍可以参考 [文件系统介绍](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter6/1fs-interface.html)

> 一句话来说，文件系统负责管理块设备储存资源，并提供从块设备上读写文件的指引

文件系统实现我使用了 rcore-fs 提供的 [sfs](https://github.com/rcore-os/rcore-fs) 文件系统，只需要实现 sfs 设备读写接口即可。

## 4.1 块设备驱动

文件系统需要从块设备上读写数据，块设备驱动负责实现该功能。按道理说，微内核的驱动也应该放在用户层，但是这里我偷懒了，直接将 qemu 块设备驱动放在内核中。由于之前开发 pipe 和 stdin stdout 时，已经定义了 File 接口，因此只需要为块设备驱动对象实现 File 的几个接口，即可将其作为 File 管理。进程可以申请打开该文件，然后通过 fd 读写文件，也就是读写块设备。

```
user/src/bin/filesystem.rs

pub fn new() -> Option<Self> {
    // 打开块设备驱动
    let fd = sys_open("qemu-blk\0");
    if fd >= 0 {
        return Some(BlkDevice { fd: Mutex::new(fd as usize) });
    } else {
        return None;
    }
}

fn read_at(&self, offset: usize, buf: &mut [u8]) -> rcore_fs::dev::Result<usize> {
    let fd = self.fd.lock().clone();
    // lseek 到指定位置，然后开始读
    sys_lseek(fd, offset);
    if sys_read(fd, buf) == (buf.len() as isize) {
        return Ok(buf.len());
    } else {
        Err(DevError)
    }
}

fn write_at(&self, offset: usize, buf: &[u8]) -> rcore_fs::dev::Result<usize> {
    let fd = self.fd.lock().clone();
    // lseek 到指定位置，然后开始写
    sys_lseek(fd, offset);
    if sys_write(fd, buf) == (buf.len() as isize) {
        return Ok(buf.len());
    } else {
        Err(DevError)
    }
}

```

## 4.2 文件的创建、读写、查询

这部分功能 sfs 已经实现好了，根据文档调用 API 即可。

目前我们没有实现文件系统缓存的功能，也就是说每次读写都需要从驱动读写，效率很低，后期可以增加一个缓存，提升效率

## 4.3 文件系统服务

文件管理和读写功能完成后，需要为其他服务提供这些功能的服务，方便其他进程读写文件。

文件系统服务使用 client - service IPC 创建各通道服务，目前提供

- 创建文件服务
- 读取文件服务

其他进程发送读取文件请求，服务端根据文件名找到文件并读取文件内容，将文件内容回复给客户端。

后续计划增加如下服务

- list 文件服务
- 写入文件服务

## 5 shell

shell 提供一个控制台，可在 shell 中运行其他进程。目前 shell 功能非常简单，甚至没有 crtl-c 中断功能，还需进一步开发。

目前只能在 shell 中运行其他进程，比如输入 hello_world 会运行 hello_world 进程

## 6 启动过程

目前的启动过程是，内核会启动初始进程，文件系统服务，文件系统服务会启动 shell，然后在 shell 中可以启动其他子进程。

后面可能会设计一个 manager 进程，类似于 systemd，启动其他进程，但是这已经和内核没什么关系了。