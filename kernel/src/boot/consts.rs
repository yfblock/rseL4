/// TODO: use dynamic 设置
pub const MAX_NUM_RESV_REG: usize = 10;
/// TODO: use dynamic 设置
pub const MAX_NUM_FREEMEM_REG: usize = 10;
pub const BOOT_INFO_FRAME_BITS: usize = 12;

#[repr(usize)]
pub enum RootCNodeCapSlots {
    Null = 0,
    InitThreadTCB = 1,
    InitThreadCNode = 2,
    InitThreadVSpace = 3,
    IRQControl = 4,
    ASIDControl = 5,
    InitThreadASIDPool = 6,
    IOPortControl = 7,
    IOSpace = 8,
    BootInfoFrame = 9,
    InitThreadIPCBuffer = 10,
    Domain = 11,
    SMMUSIDControl = 12,
    SMMUCBControl = 13,
    InitThreadSC = 14,
    SMC = 15,
    NumInitialCaps = 16,
}
