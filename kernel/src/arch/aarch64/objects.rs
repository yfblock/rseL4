use super::CONTEXT_REGS_NUM;

pub struct ArchTCB {
    context: UserContext,
}

pub struct UserContext {
    regs: [usize; CONTEXT_REGS_NUM],
}
