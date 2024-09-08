ARCH ?= riscv64

ifeq ($(ARCH), riscv64)
	TARGET = riscv64gc-unknown-none-elf
else ifeq ($(ARCH), aarch64)
	TARGET = aarch64-unknown-none
endif

MODE ?= release

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# Binutils
OBJDUMP := rust-objdump --arch-name=$(ARCH)
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)

build:
	@cargo build --target ${TARGET} $(MODE_ARG)

clean:
	@cargo clean

.PHONY: build clean