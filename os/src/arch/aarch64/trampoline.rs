/*

跳板实现的大概思路

1. 在 kernel 执行之前增加一个小的跳板程序，提供以下功能
    - 创建内核页表，用一个大页（1G）描述所有内核空间， 只考虑 64 位 cpu，内核空间地址为 0xFFFF_FFFF_0000_0000 + 内存基地址
      比如 qemu-riscv64 内核地址空间为 0xFFFF_FFFF_8000_0000，这样方便内核空间地址的转换，无需查表
    - 启动虚地址模式
    - jump 到 kernel main
2. 物理地址参数由 board 中的一个汇编代码段设置，需要传入 内核物理内存开始地址，页表基地址
3. 跳板程序会用到栈，也可能有堆，放在用户物理帧分配区域，这样不会浪费内存，会被用户数据自动覆盖

*/

use core::arch::asm;

#[no_mangle]
#[naked]
#[link_section = ".trampoline.entry"]
pub extern "C" fn __trampoline() {
  unsafe {
    asm!(
      "ldr x9, =0xff04",
      "msr MAIR_EL1, x9",
      "ldr x9, =0x4B5193519",
      "msr TCR_EL1, x9",
      "ldr x9, =0x4037F000",
      "msr TTBR1_EL1, x9",
      "msr TTBR0_EL1, x9",

      "ldr x10, =0x40000000",
      "lsr x11, x10, #30",
      "and x11, x11, #0x1FF",
      "lsl x11, x11, #3",
      "add x11, x9, x11",
      "ldr x12, =0x40000705",
      "str x12, [x11]",

      "ldr x10, =0xFFFFFFFF00000000",
      "lsr x11, x10, #30",
      "and x11, x11, #0x1FF",
      "lsl x11, x11, #3",
      "add x11, x9, x11",
      "ldr x12, =0x00000705",
      "str x12, [x11]",

      "ldr x10, =0xFFFFFFFF40000000",
      "lsr x11, x10, #30",
      "and x11, x11, #0x1FF",
      "lsl x11, x11, #3",
      "add x11, x9, x11",
      "ldr x12, =0x40000705",
      "str x12, [x11]",

      "ldr x10, =0xFFFFFFFF80000000",
      "lsr x11, x10, #30",
      "and x11, x11, #0x1FF",
      "lsl x11, x11, #3",
      "add x11, x9, x11",
      "ldr x12, =0x80000705",
      "str x12, [x11]",

      "ldr x10, =0xFFFFFFFFC0000000",
      "lsr x11, x10, #30",
      "and x11, x11, #0x1FF",
      "lsl x11, x11, #3",
      "add x11, x9, x11",
      "ldr x12, =0xC0000705",
      "str x12, [x11]",

      "isb",
      "mrs x9, SCTLR_EL1",
      "orr x9, x9, #(1 << 0)",
      "orr x9, x9, #(1 << 2)",
      "orr x9, x9, #(1 << 12)",
      "msr SCTLR_EL1, x9",
      "isb",
      "mrs x0, cpacr_el1",
      "orr x9, x9, #(0x3 << 20)",
      "msr cpacr_el1, x9",
      "adrp x0, sstack",
      "add x0, x0, :lo12:sstack",
      "mov sp, x0",
      "ldr x10, =0xFFFFFFFF41000000",
      "br x10",
      options(noreturn)
    );
  }
}
