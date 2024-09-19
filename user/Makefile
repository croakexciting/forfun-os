ARCH ?= riscv64
BOARD ?= riscv64_qemu

ifeq ($(BOARD), riscv64_qemu)
	ARCH = riscv64
else ifeq ($(BOARD), aarch64_qemu)
	ARCH = aarch64
endif

ifeq ($(ARCH), riscv64)
	TARGET = riscv64gc-unknown-none-elf
else ifeq ($(ARCH), aarch64)
	TARGET = aarch64-unknown-none
endif

MODE ?= release

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

build:
	@cargo build --target ${TARGET} $(MODE_ARG) --no-default-features --features ${ARCH}

clean:
	@cargo clean

.PHONY: build clean