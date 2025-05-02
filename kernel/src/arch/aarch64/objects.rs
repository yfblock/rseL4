use super::NUM_CONTEXT_REGS;

pub struct ArchTCB {
    context: UserContext,
}

pub struct UserContext {
    regs: [usize; NUM_CONTEXT_REGS],
}
