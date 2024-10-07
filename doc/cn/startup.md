## 1 前言

本文介绍如何编译、运行 Forfun OS

### 2 编译

clone Forfun OS

```
git clone https://github.com/croakexciting/forfun-os.git

```

编译 kernel

```
cd forfun-os

# riscv64 qemu kernel
make BOARD=riscv64_qemu build

# aarch64 qemu kernel
make BOARD=aarch6464_qemu build

```

制作文件系统

```
# riscv64 qemu fs
make BOARD=riscv64_qemu createfs

# aarch64 qemu fs
make BOARD=aarch6464_qemu createfs

```

### 3 运行

```
make BOARD=riscv64_qemu run

# or

make BOARD=aarch64_qemu run

# 进入 shell
# 运行 hello_world
>> hello_world
hello world!

# 停止 qemu
make kill

```