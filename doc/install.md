## 1 前言

本文介绍如何在本地环境安装 Forfun OS 开发和运行环境

## 2 Ubuntu 20.04/22.04

### apt 安装

安装一些依赖

```
sudo apt-get update

sudo apt-get install git build-essential gdb-multiarch qemu-system-misc gcc-riscv64-linux-gnu binutils-riscv64-linux-gnu curl autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev gawk build-essential bison flex texinfo gperf libtool patchutils bc zlib1g-dev libexpat-dev pkg-config  libglib2.0-dev libpixman-1-dev libsdl2-dev libslirp-dev tmux python3 python3-pip ninja-build
```

### Rust 安装

安装 cargo

```
curl https://sh.rustup.rs -sSf | sh

source $HOME/.cargo/env
```

如果下载速度太慢，使用国内源

```
export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
```

安装工具

```
cargo install cargo-binutils
```

### QEMU 安装

```
wget https://download.qemu.org/qemu-8.2.5.tar.xz

tar xvJf qemu-8.2.5.tar.xz

cd qemu-8.2.5

# 安装 riscv64 qemu
./configure --target-list=riscv64-softmmu,riscv64-linux-user
make -j$(nproc)
sudo make install

# 安装 aarch64 qemu
./configure --target-list=aarch64-softmmu,aarch64-linux-user
make -j$(nproc)
sudo make install
```