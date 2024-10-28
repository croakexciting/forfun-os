## 1 Introduction

This page will introduce how Forfun OS adapt to a new hardware platform. Just like the [Arch](./arch.md) part, we define a set of interfaces. You can conveniently add a hardware platform by implementing these interfaces.

These interfaces consists of the following aspects

- board initialize function, include driver, console, filesystem
- console
- shutdown
- timer relative functions
- external irq handler

## 2 Board Initialize

This function kind like the linux MACHINE_START functions, it intialize drivers, irq, filesystem and so on.

```
# aarch64 qemu board initialize
pub fn board_init() {
    CONSOLE.exclusive_access().init();
    CONSOLE.exclusive_access().ack_interrupts();
    CONSOLE.exclusive_access().is_receive_interrupt();

    GIC.exclusive_access().enable(30);
    GIC.exclusive_access().set_priority(255);

    let blk_device = BlkDeviceForFs::new(
        Arc::new(Mutex::new(QemuBlk::new(
            kernel_phys_to_virt(peripheral::BLK_HEADER_ADDR.into()).0
        ))));
    FILESYSTEM.exclusive_access().set_sfs(blk_device);
}
```

Now, I set driver base addres manually and will use device-tree config later.

## 3 Console

Board provide read and write api for console. Console is a file object in kernel. Now **read** only support block mode.

```
# CONSOLE is a Pl011 Uart driver instance

pub fn console_putchar(c: char) {
    CONSOLE.exclusive_access().putchar(c as u8)
}

pub fn console_getchar() -> u8 {
    if let Some(c) = CONSOLE.exclusive_access().getchar() {
        return c;
    } else {
        return 0;
    }
}
```

## 4 Timer

Each hardware platform has its own specific time frequency. So **Board** need to provide **set_trigger** and **nanoseconds** API.

- nanoseconds: get current CPU running in nanoseconds
- set_trigger: set next timer interrupt trigger based on the pre-defined process tick

```
# aarch64 qemu for example
pub fn nanoseconds() -> usize {
    CNTPCT_EL0.get() as usize * 1_000_000_000 / CLOCK_FREQ
}

pub fn set_trigger(tick_per_sec: usize) {
    let cntpct_el0 = CNTPCT_EL0.get() as usize;
    let new_tick = (cntpct_el0 + CLOCK_FREQ / tick_per_sec) as u64;

    CNTP_CVAL_EL0.set(new_tick);
}
```

## 5 External irq handler

Hardware drivers and relative interrupt index is different on different platform. **Board** need to provide **external_irq_handler** API for external hardware interrupt. For example, uart interrupt, block device interrupt.

The external irq handler will be called in trap handler.

```
# os/src/arch/riscv64/trap.rs:67
Trap::Interrupt(Interrupt::SupervisorExternal) => {
    crate::board::external_irq_handler();
}
```

> I think one of the best part in rust is modules can cross-reference each other. This feature can simplify source file architecture design.


## 6 Conclusion

Forfun OS provide a clear method for adapting to a new hardware platform. You can add a new platform with implementing several APIs.

Now only support two platforms, riscv64 qemu virt and aarch64 qemu virt. I will add k210 and rpi4 later.