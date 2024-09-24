TARGET ?= riscv64gc-unknown-none-elf
MODE ?= release
ARCH ?= riscv64
BOARD ?= riscv64_qemu

ifeq ($(BOARD), riscv64_qemu)
	TARGET = riscv64gc-unknown-none-elf
	ARCH = riscv64
else ifeq ($(BOARD), aarch64_qemu)
	TARGET = aarch64-unknown-none
	ARCH = aarch64
endif

.PHONY: build_kernel clean_kernel debug gdbclient \
		build_user clean_user \
		kill docker_start docker_into createfs build run clean

build_kernel:
	${MAKE} -C os build

build_user:
	${MAKE} -C user build

build: build_kernel build_user

run: build
	${MAKE} -C os run

clean_kernel:
	${MAKE} -C os clean

clean_user:
	${MAKE} -C user clean

clean: clean_kernel clean_user

debug:
	${MAKE} -C os debug

gdbclient:
	${MAKE} -C os gdbclient

createfs:
	@bash scripts/install_apps.sh ${TARGET} ${MODE} ${ARCH}
	@rm -f sfs.img
	@qemu-img create -f raw sfs.img 512M
	./tools/sfs-pack -s ./appbins/ -t ./ create

kill:
	@pkill -f qemu-system-

docker_start:
	@bash scripts/start_docker.sh

docker_into:
	@docker exec -it ffos_dev bash
