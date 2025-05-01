.equ    PT_LR,          (30 * 8)
.equ    PT_SP_EL0,      (31 * 8)
.equ    PT_ELR_EL1,     (32 * 8)
.equ    PT_SPSR_EL1,    (33 * 8)
.equ    PT_FaultIP,     (34 * 8)
.equ    PT_TPIDR_EL0,   (35 * 8)

.macro MRS_I dst, reg
    mrs     \dst, \reg\()_el1
.endm

# .macro READ_ESR dst
#     mrs     \dst, esr_el1
# .endm

# .macro READ_SP _tmp
#     mrs     \_tmp, tpidr_el1
#     mov     sp, \_tmp
# .endm

.macro ventry label
.align 7
    b       \label
.endm

.macro kernel_enter
    /* Storing thread's stack frame */
    stp     x0,  x1,  [sp, #16 * 0]
    stp     x2,  x3,  [sp, #16 * 1]
    stp     x4,  x5,  [sp, #16 * 2]
    stp     x6,  x7,  [sp, #16 * 3]
    stp     x8,  x9,  [sp, #16 * 4]
    stp     x10, x11, [sp, #16 * 5]
    stp     x12, x13, [sp, #16 * 6]
    stp     x14, x15, [sp, #16 * 7]
    stp     x16, x17, [sp, #16 * 8]
    stp     x18, x19, [sp, #16 * 9]
    stp     x20, x21, [sp, #16 * 10]
    stp     x22, x23, [sp, #16 * 11]
    stp     x24, x25, [sp, #16 * 12]
    stp     x26, x27, [sp, #16 * 13]
    stp     x28, x29, [sp, #16 * 14]

    /* Store thread's SPSR, LR, and SP */
    mrs     x21, sp_el0
    MRS_I   x22, elr
    MRS_I   x23, spsr
    stp     x30, x21, [sp, #PT_LR]
    stp     x22, x23, [sp, #PT_ELR_EL1]
.endm

.section .vectors, "ax"
BEGIN_FUNC  arm_vector_table
    ventry      invalid_vector_entry           // Synchronous EL1t/EL2t
    ventry      invalid_vector_entry           // IRQ EL1t/EL2t
    ventry      invalid_vector_entry           // FIQ EL1t/EL2t
    ventry      invalid_vector_entry           // SError EL1t/EL2t

    ventry      cur_el_sync                    // Current EL Synchronous (EL1/2)
    ventry      cur_el_irq                     // IRQ
    ventry      invalid_vector_entry           // FIQ
    ventry      cur_el_serr                    // SError

    ventry      lower_el_sync                  // Synchronous 64-bit EL0/EL1
    ventry      lower_el_irq                   // IRQ 64-bit EL0/EL1
    ventry      invalid_vector_entry           // FIQ 64-bit EL0/EL1
    ventry      lower_el_serr                  // SError 64-bit EL0/EL1

    ventry      invalid_vector_entry           // Synchronous 32-bit EL0/EL1
    ventry      invalid_vector_entry           // IRQ 32-bit EL0/EL1
    ventry      invalid_vector_entry           // FIQ 32-bit EL0/EL1
    ventry      invalid_vector_entry           // SError 32-bit EL0/EL1
END_FUNC    arm_vector_table

BEGIN_FUNC  invalid_vector_entry
    MRS_I       x19, tpidr
    b           halt
END_FUNC    invalid_vector_entry

BEGIN_FUNC  cur_el_sync
    MRS_I       x19, tpidr
    /* Read esr and branch to respective labels */
    MRS_I       x25, esr
    lsr         x24, x25, #ESR_EC_SHIFT
    cmp         x24, #ESR_EC_CEL_DABT
    b.eq        cur_el_da
    cmp         x24, #ESR_EC_CEL_IABT
    b.eq        cur_el_ia
    b           cur_el_inv

cur_el_da:
    b           halt

cur_el_ia:
    b           halt

cur_el_inv:
    b           invalid_vector_entry
END_FUNC    cur_el_sync

BEGIN_FUNC  cur_el_irq
    MRS_I       x19, tpidr
    b           c_handle_interrupt
END_FUNC    cur_el_irq

BEGIN_FUNC  cur_el_serr
    b           invalid_vector_entry
END_FUNC    cur_el_serr

BEGIN_FUNC  lower_el_sync
    kernel_enter

    /* Read esr and branch to respective labels */
    MRS_I       x25, esr
    lsr         x24, x25, #ESR_EC_SHIFT
    cmp         x24, #ESR_EC_LEL_DABT
    b.eq        lel_da
    cmp         x24, #ESR_EC_LEL_IABT
    b.eq        lel_ia
    cmp         x24, #ESR_EC_LEL_SVC64
    b.eq        lel_syscall

    cmp         x24, #ESR_EL1_EC_ENFP
    b.eq        el0_enfp
    b           el0_user

lel_da:
    MRS_I       x20, elr
    str         x20, [sp, #PT_FaultIP]

    MRS_I       x19, tpidr
    b           c_handle_data_fault

lel_ia:
    MRS_I       x20, elr
    str         x20, [sp, #PT_FaultIP]

    MRS_I       x19, tpidr
    b           c_handle_instruction_fault

lel_syscall:
    MRS_I       x20, elr
    sub         x20, x20, #4
    str         x20, [sp, #PT_FaultIP]

    MRS_I       x19, tpidr

    mov         x2, x7
    b           c_handle_syscall

el0_enfp:
# #ifdef CONFIG_HAVE_FPU
#     lsp_i   x19
#     b       c_handle_enfp
# #endif /* CONFIG_HAVE_FPU */

el0_user:
    MRS_I       x20, elr
    str         x20, [sp, #PT_FaultIP]

    MRS_I       x19, tpidr
    b           c_handle_undefined_instruction
END_FUNC    lower_el_sync

BEGIN_FUNC  lower_el_irq
    kernel_enter
    MRS_I       x20, elr
    str         x20, [sp, #PT_FaultIP]

    MRS_I       x19, tpidr
    b           c_handle_interrupt
END_FUNC    lower_el_irq

BEGIN_FUNC  lower_el_serr
    b           invalid_vector_entry
END_FUNC    lower_el_serr

BEGIN_FUNC  halt
    wfi
    b           halt
END_FUNC    halt
