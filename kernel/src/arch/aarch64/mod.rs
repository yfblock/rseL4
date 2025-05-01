#[macro_use]
mod macros;

mod boot;
mod cpu;
mod traps;
mod vspace;

/// 指向起始物理内存的虚拟地址
pub const PPTR_BASE: usize = 0xFFFF_FF80_0000_0000;
/// 指向物理内存顶的虚拟地址
pub const PPTR_TOP: usize = 0xFFFF_FFFF_C000_0000;
/// 指向内核设备内存的虚拟地址
pub const KERNEL_PT_BASE: usize = 0xFFFF_FFFF_FFE0_0000;
/// 为 [KERNEL_PT_BASE] 设置的别名
pub const KDEV_BASE: usize = KERNEL_PT_BASE;
/// VSpace 大小
pub const VSPACE_BITS: usize = 12;
/// VSpace 索引大小
pub const VSPACE_INDEX_BITS: usize = 9;
