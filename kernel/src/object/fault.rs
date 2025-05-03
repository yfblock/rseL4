/// sel4 搜索错误
///
/// ```plain
/// tagged_union lookup_fault lufType {
///     tag invalid_root 0
///     tag missing_capability 1
///     tag depth_mismatch 2
///     tag guard_mismatch 3
/// }
/// ```
pub enum LookupFault {
    InvalidRoot {},
    MissingCapability {
        bits_left: u8,
    },
    DepthMismatch {
        bits_left: u8,
        bits_found: u8,
    },
    GuardMismatch {
        bits_left: u8,
        bits_found: u8,
        guard_found: u64,
    },
}

/// sel4 错误信息
///
/// ```plain
/// tagged_union seL4_Fault seL4_FaultType {
///     -- generic faults
///     tag NullFault 0
///     tag CapFault 1
///     tag UnknownSyscall 2
///     tag UserException 3
///     -- arch specific faults
///     tag VMFault 5
/// }
/// ```
///
pub enum Fault {
    NullFault,
    CapFault {
        address: usize,
        in_receive_phase: bool,
    },
    UnknownSyscall {
        syscall_number: usize,
    },
    UserException {
        number: u32,
        code: u32,
    },
    VMFault(VMFault),
}

#[repr(usize)]
pub enum VMFault {
    DataFault,
    InstructionFault,
}

/// 任务状态类型
///
/// ```c
/// enum _thread_state {
///     ThreadState_Inactive = 0,
///     ThreadState_Running,
///     ThreadState_Restart,
///     ThreadState_BlockedOnReceive,
///     ThreadState_BlockedOnSend,
///     ThreadState_BlockedOnReply,
///     ThreadState_BlockedOnNotification,
///     ThreadState_IdleThreadState
/// };
///
/// -- Thread state: size = 24 bytes
/// block thread_state(blockingIPCBadge, blockingIPCCanGrant,
///                    blockingIPCCanGrantReply, blockingIPCIsCall,
///                    tcbQueued, blockingObject,
///                    tsType) {
///     field blockingIPCBadge 64
///
///     padding 60
///     field blockingIPCCanGrant 1
///     field blockingIPCCanGrantReply 1
///     field blockingIPCIsCall 1
///     field tcbQueued 1
///
///     padding 16
///     field_high blockingObject 44
///     field tsType 4
/// }
/// ```
#[repr(u8)]
pub enum ThreadStateType {
    InActive = 0,
    Running,
    Restart,
    BlockedOnReceive,
    BlockedOnSend,
    BlockedOnReply,
    BlockedOnNotification,
    IdleThreadState,
}

/// TODO: 根据使用状况判断是否改为 enum 判定
pub struct ThreadState {
    pub blocking_ipc_badge: u64,
    pub blocking_ipc_can_grant: bool,
    pub blocking_ipc_can_grant_reply: bool,
    pub blocking_ipc_is_call: bool,
    pub tcb_queued: bool,
    pub blocking_object: u64,
    pub ts_type: ThreadStateType,
}

/// 利用 const 静态检查断言信息
const fn _check_type_width() {
    assert!(size_of::<LookupFault>() <= 16);
    assert!(size_of::<Fault>() <= 16);
    assert!(size_of::<ThreadState>() <= 24);
}
const _: () = _check_type_width();
