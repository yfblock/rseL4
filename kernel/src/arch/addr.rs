use core::fmt::Debug;

use super::PPTR_BASE;

macro_rules! pa {
    ($addr:expr) => {
        $crate::arch::PhysAddr::new(($addr) as usize)
    };
}

macro_rules! va {
    ($addr:expr) => {
        $crate::arch::VirtAddr::new(($addr) as usize)
    };
}

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct PhysAddr(usize);
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct VirtAddr(usize);

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct VirtRegion {
    pub start: VirtAddr,
    pub end: VirtAddr,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct PhysRegion {
    pub start: PhysAddr,
    pub end: PhysAddr,
}

macro_rules! impl_addr {
    ($($name:ident),*) => {
        $(
            impl $name {
                pub const fn new(addr: usize) -> Self {
                    Self(addr)
                }

                pub const fn raw(&self) -> usize {
                    self.0
                }
            }

            impl From<$name> for usize {
                fn from(addr: $name) -> Self {
                    addr.0
                }
            }

            impl From<$name> for u64 {
                fn from(addr: $name) -> Self {
                    addr.0 as u64
                }
            }

            impl core::fmt::Debug for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_fmt(format_args!("{}({:#x})", stringify!($name), self.0))
                }
            }

            impl core::ops::Add<usize> for $name {
                type Output = Self;

                fn add(self, rhs: usize) -> Self::Output {
                    Self(self.0 + rhs)
                }
            }
        )*
    };
}

macro_rules! impl_addr_range {
    ($($name:ident($ty: ident)),*) => {
        $(
            impl $name {
                pub const fn new(start: $ty, end: $ty) -> Self {
                    Self { start, end }
                }
                pub const fn empty() -> Self {
                    Self {
                        start: $ty::new(0),
                        end: $ty::new(0)
                    }
                }
                pub const fn is_empty(&self) -> bool {
                    self.start.raw() == 0 && self.end.raw() == 0
                }
            }
        )*
    };
}

impl_addr!(PhysAddr, VirtAddr);
impl_addr_range!(PhysRegion(PhysAddr), VirtRegion(VirtAddr));

impl PhysAddr {
    pub const fn pptr(&self) -> VirtAddr {
        VirtAddr(self.0 | PPTR_BASE)
    }
}

impl PhysRegion {
    pub const fn pptr(&self) -> VirtRegion {
        VirtRegion {
            start: self.start.pptr(),
            end: self.end.pptr(),
        }
    }
}

impl VirtRegion {
    pub const fn paddr(&self) -> PhysRegion {
        PhysRegion {
            start: pa!(self.start.0),
            end: pa!(self.end.0),
        }
    }
}
