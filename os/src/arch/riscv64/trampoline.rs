use core::arch::asm;

#[no_mangle]
#[naked]
#[link_section = ".trampoline.entry"]
pub extern "C" fn __trampoline() {
    unsafe {
        asm!(
            // Set Identical map for trampoline
            "la a0, skpt",
            "la a1, strampoline",
            "li t0, 0x7FC0000000",
            "and a2, a1, t0",
            "srli a2, a2, 30",
            "slli a2, a2, 3",
            "add t2, a0, a2",
            "srli a1, a1, 30",
            "slli a1, a1, 28",
            "ori a1, a1, 0xF",
            "sd a1, 0(t2)",

            // Set 0xFFFFFFFF00000000 ~ 0xFFFFFFFFFFFFFFFF for kernel space
            "li t0, 0x7FC0000000",
            "li a1, 0xFFFFFFFF00000000",
            "and a2, a1, t0",
            "srli a2, a2, 30",
            "slli a2, a2, 3",
            "add t2, a0, a2",
            "li t0, 0xF",
            "sd t0, 0(t2)",

            "li t0, 0x7FC0000000",
            "li a1, 0xFFFFFFFF40000000",
            "and a2, a1, t0",
            "srli a2, a2, 30",
            "slli a2, a2, 3",
            "add t2, a0, a2",
            "li t0, 0x1000000F",
            "sd t0, 0(t2)",

            "li t0, 0x7FC0000000",
            "li a1, 0xFFFFFFFF80000000",
            "and a2, a1, t0",
            "srli a2, a2, 30",
            "slli a2, a2, 3",
            "add t2, a0, a2",
            "li t0, 0x2000000F",
            "sd t0, 0(t2)",

            "li t0, 0x7FC0000000",
            "li a1, 0xFFFFFFFFC0000000",
            "and a2, a1, t0",
            "srli a2, a2, 30",
            "slli a2, a2, 3",
            "add t2, a0, a2",
            "li t0, 0x3000000F",
            "sd t0, 0(t2)",

            // enable virtual addr
            "srli a0, a0, 12",
            "li t0, 0x8000000000000000",
            "or a0, a0, t0",
            "csrw satp, a0",
            "sfence.vma",
            "lui t0, %hi(sstack)",
            "addi t0, t0, %lo(sstack)",
            "mv sp, t0",
            "lui t0, %hi(os_main)",
            "addi t0, t0, %lo(os_main)",
            "jalr t1, t0, 0",
            options(noreturn)
        )
    }
}