TARGET ?= riscv64gc-unknown-none-elf
MODE ?= release

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

APP_ENTRY := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

build:
	@cargo build --target ${TARGET} $(MODE_ARG)

clean:
	@cargo clean

.PHONY: build clean