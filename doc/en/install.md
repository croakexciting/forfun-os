## 1 Introduction

This page will tell you how to install forfun os dev environment. 

Now I only test it on ubuntu20.04/22.04, use wsl on windows also tested.

## 2 Ubuntu 20.04/22.04

### Install apt packages

```
sudo apt-get update

sudo apt-get install git build-essential gdb-multiarch qemu-system-misc gcc-riscv64-linux-gnu binutils-riscv64-linux-gnu curl autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev gawk build-essential bison flex texinfo gperf libtool patchutils bc zlib1g-dev libexpat-dev pkg-config  libglib2.0-dev libpixman-1-dev libsdl2-dev libslirp-dev tmux python3 python3-pip ninja-build
```

### Install rust stuff

```
curl https://sh.rustup.rs -sSf | sh

source $HOME/.cargo/env

cargo install cargo-binutils
```

### Install QEMU

```
wget https://download.qemu.org/qemu-8.2.5.tar.xz

tar xvJf qemu-8.2.5.tar.xz

cd qemu-8.2.5

# Install riscv64 qemu
./configure --target-list=riscv64-softmmu,riscv64-linux-user
make -j$(nproc)
sudo make install

# Install aarch64 qemu
./configure --target-list=aarch64-softmmu,aarch64-linux-user
make -j$(nproc)
sudo make install
```