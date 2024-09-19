    .section .text.entry
    .globl  _start
_start:
    mrs     x0, cpacr_el1
    orr     x9, x9, #(0x3 << 20)
    msr     cpacr_el1, x9
    isb
    adrp    x0, sstack
    add     x0, x0, :lo12:sstack
    mov     sp, x0
    b       os_main
