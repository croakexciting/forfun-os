// page table manager

// Every app has it's own page table
struct PageTable {
    root: riscv::register::satp::Satp,
}