pub struct Irq(pub usize);

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum IrqState {
    IRQInactive = 0,
    IRQSignal = 1,
    IRQTimer = 2,
    IRQReserved,
}
