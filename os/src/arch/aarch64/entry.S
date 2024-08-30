    .section .text.entry
    .globl  _start
_start:
    ADRP    x0, sstack
    ADD     x0, x0, :lo12:sstack
    MOV     sp, x0
    B       os_main
