<h2 align="center">Forfun OS</h2>

<p align="center">
   <a href="README.md">English</a>
</p>

## 1 前言

在学习 rcore 之后，为了更加了解其中的细节，萌生了自己写一个操作系统练练手的想法。这个操作系统只是用于个人学习所用，所有设计都是简化处理。虽然简单，但是一个完整的类 unix 操作系统，基本功能均完成。

<img src="./drawio/architecture.svg" alt="项目架构"/>

## 1.1 详细设计文档

- [系统调用及任务调度](./doc/cn/syscall.md)
- [内存管理](./doc/cn/memory.md)
- [进程管理和 IPC](./doc/cn/process.md)
- [用户程序开发](./doc/cn/user.md)

## 2 功能和开发计划

### 2.1 已完成功能

* [x] Syscall
* [x] Task schedule (Round Robin)
* [x] Memory and MMU manager
* [x] Process manager and IPCs
* [x] Simple filesystem
* [x] CPUs (riscv64, aarch64)
* [x] Boards (qemu virt)
* [x] Virtio blk driver

### 2.2 TODOs

* [ ] Board support (k210, rpi4)
* [ ] Thread
* [ ] Network driver and TCP/UDP stack
* [ ] Multi-core
* [ ] Driver Interrupt
* [ ] Linux app adaptation


## 3 快速开始

### 3.1 安装环境

为了方便体验，可以使用 docker 进行运行开发，请预先安装 docker 

另外可以本地安装，安装文档 [本地环境安装](./doc/cn/install.md)

### 3.2 运行 Forfun OS

> 请确认已安装 docker

```
make docker_start

make docker_into

# default run on riscv64 qemu
make run

# 运行正常，会进入 shell
# 运行 hello_world
>> hello_world
hello world!

# 停止内核，目前必须用 kill 才可以停止
make kill
```

[编译和运行文档](./doc/cn/startup.md)

### 3.3 可以运行的用户程序

- hello_world
- loop_test
- sleep_test

## 结语

由于操作系统和编译器等软件基础工具封装的太好了，为软件开发人员提供了一个统一个开发环境，其实他们内在是存在很多与硬件相互协作的过程。这当然是一件好事，但是也造成了软件开发人员对于底层理解不多。如果想要开发更好地软件，还是需要加深对操作系统和编译器的理解。当然这部分知识太多了，没有人可以全部掌握，我们需要的是了解其背后的基本原理，至于各种细节倒不必花太多时间。当理解其背后的原理后，很多之前无法理解的概念就豁然开朗了

另外，本人水平有限，对于操作系统开发更是所知不多，如果您发现有错漏之处，还请不吝赐教。

欢迎提 issue，PR，issue 甚至可以只是一条评论，这只是一个玩具项目，所以请放轻松😊
