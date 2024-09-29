OUTPUT_ARCH(aarch64)
ENTRY(__trampoline)
_heap_size = 0x100000;
/* leave 512K for app kernel stack */
_stack_size = 0x10000;

MEMORY
{
    TRAMPOLINE  : ORIGIN = 0x40000000,          LENGTH = 16K
    VIRTUAL     : ORIGIN = 0xFFFFFFFF41000000,  LENGTH = 8M
}

SECTIONS
{
    .trampoline ALIGN(4K) :
    {
        strampoline = .;
        KEEP(*(.trampoline.entry))
        etrampoline = .;
    } > TRAMPOLINE

    .text ALIGN(4K) : AT(0x41000000)
    {
        KEEP(*(.text.entry))
        *(.text .text.*)
    } > VIRTUAL

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } > VIRTUAL

    .data : ALIGN(4K)
    {
        *(.data .data.*)
        *(.sdata .sdata.*)
    } > VIRTUAL

    .bss : ALIGN(4K)
    {
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    } > VIRTUAL
    ebss = .;

    /* reserve 1M space for kernel heap*/
    .heap (NOLOAD) : ALIGN(4K)
    {
        sheap = .;
        . += _heap_size;
        eheap = .;
    } > VIRTUAL

    /* stack grow from high address to low address */
    .stack (NOLOAD) : ALIGN(4K)
    {
        estack = .;
        . += _stack_size;
        sstack = .;
    } > VIRTUAL

    /DISCARD/ : {
        *(.eh_frame)
    }
}