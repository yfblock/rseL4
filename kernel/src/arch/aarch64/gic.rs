use crate::{
    arch::{irq::Irq, IrqState},
    boot::consts::RootCNodeCapSlots,
    driver::set_irq_state,
    object::{cspace::CNode, structures::IrqControlCap},
    platform::{KERNEL_TIMER_IRQ, MAX_IRQ},
};

impl Irq {
    pub const fn new(_core: usize, irq: usize) -> Self {
        Self(irq)
    }
}

pub fn init_irqs(root_cnode_cap: &mut CNode) {
    for i in 0..MAX_IRQ {
        set_irq_state(IrqState::IRQInactive, Irq::new(0, i));
    }
    set_irq_state(IrqState::IRQTimer, Irq::new(0, KERNEL_TIMER_IRQ));
    // TODO: Init IRQTimer
    root_cnode_cap.write(RootCNodeCapSlots::IRQControl as _, &IrqControlCap::empty());
}
