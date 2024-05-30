OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80020000;
_heap_size = 0x100000;
_stack_start = 0x801FFFF0;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    .bss : {
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;

    /* reserve 1M space for kernel heap*/
    .heap (NOLOAD) :
    {
        sheap = .;
        . += _heap_size;
        . = ALIGN(4);
        eheap = .;
    }

    /* stack grow from high address to low address */
    .stack (NOLOAD) :
    {
        estack = .;
        . = ABSOLUTE(_stack_start);
        sstack = .;
    }

    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}