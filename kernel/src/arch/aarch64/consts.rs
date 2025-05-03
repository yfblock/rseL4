/// origin：`#define seL4_PageBits 12`
/// 页（4 KiB）对象大小的比特位数, 用于表示页大小。
pub const PAGE_BITS: usize = 12;

/// origin：`#define seL4_LargePageBits 21`
/// 大页（2 MiB）对象大小的比特位数, 用于支持大页内存映射。
pub const LARGE_PAGE_BITS: usize = 21;

/// origin：`#define seL4_HugePageBits 30`
/// 巨页（1 GiB）对象大小的比特位数, 用于支持巨页内存映射。
pub const HUGE_PAGE_BITS: usize = 30;

/// origin：`#define seL4_SlotBits 5`
/// CNode 插槽大小的比特位数, 用于描述 CNode 内部的插槽大小。
pub const SLOT_BITS: usize = 5;

/// origin：`#define seL4_TCBBits 11`
/// TCB（线程控制块）对象大小的比特位数, 用于描述线程控制块（TCB）的大小。
pub const TCB_BITS: usize = 11;

/// origin：`#define seL4_EndpointBits 4`
/// Endpoint（通信端点）对象大小的比特位数, 用于管理线程之间的同步与通信。
pub const ENDPOINT_BITS: usize = 4;

/// origin：`#define seL4_NotificationBits 5`
/// Notification（通知）对象大小的比特位数, 用于在多个进程或线程之间进行消息通知。
pub const NOTIFICATION_BITS: usize = 5;

/// origin：`#define seL4_PageTableBits 12`
/// 页表对象大小的比特位数, 用于定义页表大小，用于虚拟内存管理。
pub const PAGE_TABLE_BITS: usize = 12;

/// origin：`#define seL4_PageTableEntryBits 3`
/// 页表项大小的比特位数, 用于定义页表项的大小，描述页表中每个条目的结构。
pub const PAGE_TABLE_ENTRY_BITS: usize = 3;

/// origin：`#define seL4_PageTableIndexBits 9`
/// 页表索引的比特位数, 用于在页表中查找具体条目时，指定索引的大小。
pub const PAGE_TABLE_INDEX_BITS: usize = 9;

/// origin：`#define seL4_NumASIDPoolsBits 7`
/// 支持的 ASID 池数量的比特位数, 用于管理 ASID 池的数量，ASID 池用于区分不同的地址空间。
pub const NUM_ASID_POOLS_BITS: usize = 7;

/// origin：`#define seL4_ASIDPoolBits 12`
/// 单个 ASID 池对象大小的比特位数, 用于定义每个 ASID 池的大小。
pub const ASID_POOL_BITS: usize = 12;

/// origin：`#define seL4_ASIDPoolIndexBits 9`
/// ASID 池索引的比特位数, 用于在 ASID 池中查找特定索引。
pub const ASID_POOL_INDEX_BITS: usize = 9;

/// origin：`#define seL4_IOPageTableBits 12`
/// IO 页表对象大小的比特位数, 用于定义用于输入输出操作的页表大小。
pub const IO_PAGE_TABLE_BITS: usize = 12;

/// origin：`#define seL4_WordSizeBits 3`
/// 一个 word（字长）的大小（按位）偏移, 用于表示平台架构中字长的位宽，影响指针和数据的表示方式。
pub const WORD_SIZE_BITS: usize = 3;

/// origin：`#define seL4_VSpaceEntryBits 3`
/// VSpace 表项的比特位数, 用于虚拟地址空间（VSpace）的映射条目大小。
pub const VSPACE_ENTRY_BITS: usize = 3;

/// origin：`#define seL4_VSpaceBits 12`
/// VSpace 对象大小的比特位数, 用于表示虚拟空间的大小，影响虚拟地址的分配。
pub const VSPACE_BITS: usize = 12;

/// origin：`#define seL4_VSpaceIndexBits 9`
/// VSpace 索引比特数, 用于索引虚拟地址空间中的条目。
pub const VSPACE_INDEX_BITS: usize = 9;

/// origin：`#define seL4_ARM_VCPUBits 12`
/// ARM 架构 VCPU 对象大小的比特位数, 用于定义 ARM 架构下虚拟 CPU 的大小。
pub const ARM_VCPU_BITS: usize = 12;

/// origin：`#define seL4_VCPUBits 12`
/// 通用 VCPU 对象大小的比特位数, 用于定义虚拟 CPU 的大小，适用于虚拟化系统中的 CPU 模拟。
pub const VCPU_BITS: usize = 12;

/// origin：`#define seL4_WordBits 64`
/// 系统中一个 word 的位宽, 用于描述平台的基本数据类型字长，影响指针和数据的表示方式。
pub const WORD_BITS: usize = 64;

/// origin：`#define seL4_MinUntypedBits 4`
/// 最小 Untyped 对象的比特位数, 用于表示内存中未类型化对象的最小大小。
pub const MIN_UNTYPED_BITS: usize = 4;

/// origin：`#define seL4_MaxUntypedBits 47`
/// 最大 Untyped 对象的比特位数, 用于表示内存中未类型化对象的最大大小。
pub const MAX_UNTYPED_BITS: usize = 47;

/// 初始任务的 ASID
pub const IT_ASID: usize = 1;
