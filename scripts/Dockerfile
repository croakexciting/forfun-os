FROM ubuntu:20.04 as build_qemu

ARG QEMU_VERSION=7.0.0

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y wget build-essential python3 libglib2.0-dev libfdt-dev libpixman-1-dev zlib1g-dev ninja-build

RUN wget https://download.qemu.org/qemu-${QEMU_VERSION}.tar.xz && \
    tar xf qemu-${QEMU_VERSION}.tar.xz && \
    cd qemu-${QEMU_VERSION} && \ 
    ./configure --target-list=riscv64-softmmu,riscv64-linux-user && \
    make -j$(nproc) && \
    make install

FROM ubuntu:20.04 as ffos_dev

# 2.0. Install general tools
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y jq curl git python3 wget build-essential \
    # qemu dependency
    libglib2.0-0 libfdt1 libpixman-1-0 zlib1g \
    # gdb
    gdb-multiarch

# 2.1. Copy qemu
COPY --from=build_qemu /usr/local/bin/* /usr/local/bin

# 2.2. Install Rust
# - https://www.rust-lang.org/tools/install
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static \
    RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup 
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --no-modify-path --profile minimal --default-toolchain nightly

ARG RUST_VERSION=nightly-2024-05-20

RUN rustup target add riscv64gc-unknown-none-elf && \
    cargo install cargo-binutils

# 2.4. Set GDB
RUN ln -s /usr/bin/gdb-multiarch /usr/bin/riscv64-unknown-elf-gdb
