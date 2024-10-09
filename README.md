<h2 align="center">Forfun OS</h2>

<p align="center">
   <a href="README_CN.md">中文</a>
</p>

## 1 Introduction

Forfun OS is a unix-like kernel, written by rust. Now support riscv64 and aarch64 CPU architectures. It is an simple OS just for os learning, only complete basic functions.

<img src="./drawio/architecture.svg" alt="kernel architecture"/>

### 1.1 Design Docs (ongoing)

- [Syscall](./doc/en/syscall.md)
- [Task schedule]()
- [Memory manager]()
- [Process manager and IPC]()
- [User process development]()

## 2 Features and TODOs

### 2.1 Features

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

## 3 Quick Start

Install the dev environment ref to [installation guide](./doc/en/install.md)

You can also use docker for quick start

```
make docker_start

make docker_into

```

Build and run forfun os ref to [Build and run](./doc/en/startup.md)

```
# default run on riscv64 qemu virt
make build

make createfs

make run

# use BOARD arg change target board
make BOARD=aarch64_qemu build

make BOARD=aarch64_qemu createfs

make BOARD=aarch64_qemu run

```

Run some app on the very simple shell

```
# run hello_world
>> hello_world
hello world!

# run sleep_test
>> sleep_test

# use crtl-c to stop unterminated app

```

## Contact me

Forfun os is just a toy project and not tested, if you find any bug or have any idea and comment, please create a issue or pull request.
