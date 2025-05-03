#[macro_use]
mod macros;

mod boot;
mod consts;
mod cpu;
mod gic;
mod mem;
mod objects;
mod traps;
mod vspace;

pub use consts::*;
pub use mem::arch_get_n_paging;
pub use objects::{ArchTCB, UserContext};
pub use vspace::VSpace;

const NUM_CONTEXT_REGS: usize = 37;

const NUM_RESERVED_REGIONS: usize = 12;

const PAGE_SIZE: usize = bit!(PAGE_BITS);

/// 指向起始物理内存的虚拟地址
pub const PPTR_BASE: usize = 0xFFFF_FF80_0000_0000;
/// 指向物理内存顶的虚拟地址
pub const PPTR_TOP: usize = 0xFFFF_FFFF_C000_0000;
/// 指向内核设备内存的虚拟地址
pub const KERNEL_PT_BASE: usize = 0xFFFF_FFFF_FFE0_0000;
/// 为 [KERNEL_PT_BASE] 设置的别名
pub const KDEV_BASE: usize = KERNEL_PT_BASE;
