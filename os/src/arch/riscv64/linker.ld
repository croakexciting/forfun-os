OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;
_heap_size = 0xD0000;
/* leave 512K for app kernel stack */
_stack_start = 0x80380000;

MEMORY
{
    SRAM : ORIGIN = 0x80000000, LENGTH = 6M
    AI_SRAM : ORIGIN = 0x80600000, LENGTH = 2M
    SRAM_NOCACHE : ORIGIN = 0x40000000, LENGTH = 6M
    AI_SRAM_NOCACHE : ORIGIN = 0x40600000, LENGTH = 2M
}

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    .trampoline : ALIGN(4)
    {
        /* Add a dummy symbol or address marker here */
        _dummy_start = .;
        *(.trampoline.start)
        _dummy_end = .;
    } > SRAM

    stext = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    } > SRAM

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } > SRAM

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    } > SRAM

    . = ALIGN(4K);
    edata = .;
    .bss : {
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    } > SRAM

    . = ALIGN(4K);
    ebss = .;

    /* reserve 1M space for kernel heap*/
    .heap (NOLOAD) :
    {
        sheap = .;
        . += _heap_size;
        . = ALIGN(4K);
        eheap = .;
    } > SRAM

    /* stack grow from high address to low address */
    .stack (NOLOAD) :
    {
        estack = .;
        . = ABSOLUTE(_stack_start);
        sstack = .;
    } > SRAM

    . = ALIGN(4K);
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}