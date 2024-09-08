ARCH ?= riscv64
BOARD ?= riscv64_qemu

ifeq ($(BOARD), riscv64_qemu)
	ARCH = riscv64
else ifeq ($(BOARD), aarch64_qemu)
	ARCH = aarch64
endif

ifeq ($(ARCH), riscv64)
	TARGET = riscv64gc-unknown-none-elf
	KERNEL_ENTRY = 0x80200000
	APP_ENTRY = 0x81000000
else ifeq ($(ARCH), aarch64)
	TARGET = aarch64-unknown-none
	KERNEL_ENTRY = 0x40000000
	APP_ENTRY = 0x41000000
endif

MODE ?= release
KERNEL_ELF := target/$(TARGET)/$(MODE)/forfun-os
KERNEL_BIN := $(KERNEL_ELF).bin
APP_BIN := ../user/target/$(TARGET)/$(MODE)/hello_world

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# Binutils
OBJDUMP := rust-objdump --arch-name=$(ARCH)
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)

ifeq ($(ARCH), riscv64)
	QEMU_ARGS = -machine virt \
			 -nographic \
			 -bios ../bootloader/rustsbi-qemu.bin \
			 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY) \
			 -device loader,file=$(APP_BIN),addr=$(APP_ENTRY),force-raw=on \
			 -drive file=../sfs.img,if=none,format=raw,id=x0 \
			 -device virtio-blk-device,drive=x0
else ifeq ($(ARCH), aarch64)
	QEMU_ARGS = -machine virt \
			 -cpu cortex-a72 \
			 -nographic \
			 -serial mon:stdio \
			 -kernel $(KERNEL_ELF) \
			 -device loader,file=$(APP_BIN),addr=$(APP_ENTRY),force-raw=on
endif

build:
	@echo Platform: $(BOARD)
	@cargo build --target ${TARGET} $(MODE_ARG) --no-default-features --features ${BOARD}
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary ${KERNEL_BIN}

clean:
	@cargo clean

run:
ifeq ($(findstring qemu, $(BOARD)), qemu)
	@qemu-system-$(ARCH) $(QEMU_ARGS)
endif

debug: build
	@qemu-system-$(ARCH) $(QEMU_ARGS) -s -S

gdbclient:
ifeq ($(ARCH), riscv64)
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'
else ifeq ($(ARCH), aarch64)
	@aarch64-none-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'target remote localhost:1234'
endif

.PHONY: build clean run debug gdbclient