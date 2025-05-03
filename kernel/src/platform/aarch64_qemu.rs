use crate::arch::{PhysRegion, VAddr, PPTR_BASE, PPTR_TOP};

/// 平台物理内存起始地址
pub const PADDR_BASE: usize = 0;
pub const PADDR_ELF_ADDR: usize = 0x4000_0000;
/// 物理内存 TOP 地址
pub const PADDR_TOP: usize = PPTR_TOP - PPTR_BASE;

pub const USER_TOP: VAddr = va!(0xa0000000);

pub const PLAT_MEM_REGIONS: &[PhysRegion] = &[PhysRegion::new(pa!(0x4000_0000), pa!(0x8000_0000))];

pub const MAX_IRQ: usize = 159;
pub const TIMER_FREQUENCY: usize = 62500000;

pub const KERNEL_TIMER_IRQ: usize = 1;
