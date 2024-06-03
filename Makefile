TARGET ?= riscv64gc-unknown-none-elf
MODE ?= release
KERNEL_ELF := target/$(TARGET)/$(MODE)/forfun-os
KERNEL_BIN := $(KERNEL_ELF).bin
APP_BIN := appbins/hello_world.bin

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# board
BOARD ?= qemu
SBI ?= rustsbi
BOOTLOADER := ./bootloader/$(SBI)-$(BOARD).bin

# k210 setting
K210-SERIALPORT ?= /dev/ttyUSB0
K210-BOARD ?= kd233
K210_BOOTLOADER_SIZE := 131072

KERNEL_ENTRY := 0x80020000
APP_ENTRY := 0x80200000
APP_ENTRY2 := 0x80300000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

build:
	@echo Platform: $(BOARD)
	@cargo build --target ${TARGET} $(MODE_ARG)
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary ${KERNEL_BIN}

clean:
	@cargo clean

QEMU_ARGS := -machine virt \
			 -nographic \
			 -bios $(BOOTLOADER) \
			 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY) \
			 -device loader,file=$(APP_BIN),addr=$(APP_ENTRY) \
			 -device loader,file=$(APP_BIN),addr=$(APP_ENTRY2)

run: build
ifeq ($(BOARD), qemu)
	@qemu-system-riscv64 $(QEMU_ARGS)
else ifeq ($(BOARD), k210)
	@cp $(BOOTLOADER) $(BOOTLOADER).copy
	@dd if=$(KERNEL_BIN) of=$(BOOTLOADER).copy bs=$(K210_BOOTLOADER_SIZE) seek=1
	@mv $(BOOTLOADER).copy $(KERNEL_BIN)
	python3 tools/kflash/kflash.py -p $(K210-SERIALPORT) -b 1500000 -B $(K210-BOARD) -t $(KERNEL_BIN)
endif

debug: build
	qemu-system-riscv64 $(QEMU_ARGS) -s -S

.PHONY: build clean run