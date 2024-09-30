_heap_size = 0x200000;
_stack_size = 0x10000;

MEMORY
{
    TRAMPOLINE  : ORIGIN = 0x40000000,          LENGTH = 8K
    VIRTUAL     : ORIGIN = 0xFFFFFFFF40002000,  LENGTH = 4M
}

INCLUDE src/arch/aarch64/linker.ld.in