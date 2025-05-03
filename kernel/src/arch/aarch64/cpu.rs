use super::vspace::activate_kernel_vspace;
use crate::{
    arch::{generic::KERNEL_STACK_ALLOC, PAddr, PPTR_BASE},
    config::KERNEL_STACK_BITS,
    driver::{init_irq_controller, init_timer},
};
use aarch64_cpu::{
    asm::barrier::{self, dsb, isb},
    registers::{Writeable, CNTKCTL_EL1, TPIDR_EL1, VBAR_EL1},
};

pub fn init_cpu() {
    activate_kernel_vspace();
    unsafe {
        let stack_top = (&raw mut KERNEL_STACK_ALLOC[0]) as usize + bit!(KERNEL_STACK_BITS);
        TPIDR_EL1.set(stack_top as _);
        extern "C" {
            fn arm_vector_table();
        }
        set_vtable(pa!(arm_vector_table as usize - PPTR_BASE));
    }
    crate::driver::cpu_init_local_irq_controller();
    armv_init_user_access();
    init_timer();
}

pub fn init_plat() {
    init_irq_controller();
}

pub fn set_vtable(paddr: PAddr) {
    assert!(paddr.raw() % 4 == 0);
    dsb(barrier::SY);
    VBAR_EL1.set(paddr.raw() as _);
    isb(barrier::SY);
}

pub fn armv_init_user_access() {
    // 暴露 Physical Timer 给用户态
    CNTKCTL_EL1.write(CNTKCTL_EL1::EL0PCTEN::SET + CNTKCTL_EL1::EL0PTEN::SET);
}
