# Forfun OS

## 关于项目

该项目是一个自己实现的类 unix 操作系统**玩具**。主要目的是方便个人学习操作系统，加深对软件的理解。

[设计文档](https://croak.cxyz.space/posts/ffos/)

## 特性

已经实现的特性如下，由于按照微内核设计，因此文件系统放在了用户侧

- Kernel
    - Arch
        - aarch64
        - riscv64
    - Board
        - QEMU Virt
    - 用户层系统调用
    - 任务调度
    - 虚拟内存管理
    - IPC
        - 信号量
        - 共享内存
        - 管道
        - 命名信号
        - client-service
    - 文件系统适配

- User
    - Shell
    - hello_world
    - loop_test
    - sleep_test

## 快速开始

为了方便体验，可以使用 docker 进行运行开发，步骤如下

> 请确认已安装 docker

```
make docker_start

make docker_into

# default run on riscv64 qemu
make run

# run hello_world user example 
>> hello_world

# stop kernel
make kill
```

如果不想在 docker 中进行开发，环境安装请参考
https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html

## 说明

本项目是一个操作系统学习作业，本人也不是专业的 os 开发人员，对于 os 的理解也不深，因此可能存在较多 bug 和设计不合理的地方。

如果您发现了 bug，欢迎提 issue，或者直接提 PR，谢谢！
