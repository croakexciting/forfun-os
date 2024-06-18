    .section .text.entry
    .globl _start
_start:
    la sp, sstack
    call os_main