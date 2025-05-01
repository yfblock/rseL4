use crate::arch::{PPTR_BASE, PPTR_TOP};

/// 平台物理内存起始地址
pub const PADDR_BASE: usize = 0;
/// 物理内存 TOP 地址
pub const PADDR_TOP: usize = PPTR_TOP - PPTR_BASE;
