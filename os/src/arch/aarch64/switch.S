    .section .text
    .globl  __switch
__switch:
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )
    # save callee register

    mov     x9, sp
    stp     lr, x9, [x0, #16 * 0]
    stp     x19, x20, [x0, #16 * 1]
    stp     x21, x22, [x0, #16 * 2]
    stp     x23, x24, [x0, #16 * 3]
    stp     x25, x26, [x0, #16 * 4]
    stp     x27, x28, [x0, #16 * 5]
    str     x29,      [x0, #16 * 6]
    # restore callee register
    ldp     lr, x9, [x1, #16 * 0]
    ldp     x19, x20, [x1, #16 * 1]
    ldp     x21, x22, [x1, #16 * 2]
    ldp     x23, x24, [x1, #16 * 3]
    ldp     x25, x26, [x1, #16 * 4]
    ldp     x27, x28, [x1, #16 * 5]
    ldr     x29,      [x1, #16 * 6]
    mov     sp, x9
    ret