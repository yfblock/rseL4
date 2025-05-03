use crate::arch::{Irq, IrqState};

pub fn cpu_init_local_irq_controller() {}

pub fn init_irq_controller() {}

pub fn set_irq_state(state: IrqState, irq: Irq) {
    if state != IrqState::IRQInactive {
        log::warn!("need to implement state: {:?} @ {:#x}", state, irq.0);
    }
}
