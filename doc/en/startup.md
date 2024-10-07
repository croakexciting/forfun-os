## 1 Build

Clone Forfun os

```
git clone https://github.com/croakexciting/forfun-os.git

```

Build kernel

```
cd forfun-os

# riscv64 qemu kernel
make BOARD=riscv64_qemu build

# aarch64 qemu kernel
make BOARD=aarch6464_qemu build

```

Make filesystem

```
# riscv64 qemu fs
make BOARD=riscv64_qemu createfs

# aarch64 qemu fs
make BOARD=aarch6464_qemu createfs

```

## 2 Run

```
make BOARD=riscv64_qemu run

# or

make BOARD=aarch64_qemu run

# Enter shell
# Run hello_world
>> hello_world
hello world!

# run sleep_test
>> sleep_test

# use crtl-c to stop unterminated app

# stop qemu
make kill

```

## 3 Debug

Need to terminal window

```
# Window 1

make debug

# Window 2

make gdbclient

```

