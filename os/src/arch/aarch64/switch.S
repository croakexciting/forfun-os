    .section .text
    .globl  __switch
__switch:
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )
    # save callee register
    stp     lr, sp, [a0, #16 * 0]
    stp     x19, x20, [a0, #16 * 1]
    stp     x21, x22, [a0, #16 * 2]
    stp     x23, x24, [a0, #16 * 3]
    stp     x25, x26, [a0, #16 * 4]
    stp     x27, x28, [a0, #16 * 5]
    stp     x29, x30, [a0, #16 * 6]
    # restore callee register
    ldp     lr, sp, [a1, #16 * 0]
    ldp     x19, x20, [a1, #16 * 1]
    ldp     x21, x22, [a1, #16 * 2]
    ldp     x23, x24, [a1, #16 * 3]
    ldp     x25, x26, [a1, #16 * 4]
    ldp     x27, x28, [a1, #16 * 5]
    ldp     x29, x30, [a1, #16 * 6]
    ret