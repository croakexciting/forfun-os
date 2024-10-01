_heap_size = 0x100000;
_stack_size = 0x10000;
_load_addr = 0x80202000;

MEMORY
{
    TRAMPOLINE  : ORIGIN = 0x80200000,          LENGTH = 8K
    VIRTUAL     : ORIGIN = 0xFFFFFFFF80202000,  LENGTH = 4M
}

INCLUDE src/arch/riscv64/linker.ld.in