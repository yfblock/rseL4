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

                pub const fn align_up(&self, bits: usize) -> Self {
                    Self(self.0.div_ceil(bit!(bits)) * bit!(bits))
                }
                pub const fn align_down(&self, bits: usize) -> Self {
                    Self(self.0 & !(bit!(bits) - 1))
                }
                pub const fn is_null(&self) -> bool {
                    self.0 == 0
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
            impl core::fmt::Display for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_fmt(format_args!("{:#x}", self.0))
                }
            }
            impl core::fmt::LowerHex for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    core::fmt::LowerHex::fmt(&self.0, f)
                }
            }
            impl core::ops::Add<usize> for $name {
                type Output = Self;

                fn add(self, rhs: usize) -> Self::Output {
                    Self(self.0 + rhs)
                }
            }
            impl core::ops::Sub<usize> for $name {
                type Output = Self;

                fn sub(self, rhs: usize) -> Self::Output {
                    Self(self.0 - rhs)
                }
            }
            impl core::ops::BitAnd<usize> for $name {
                type Output = Self;

                fn bitand(self, rhs: usize) -> Self::Output {
                    Self(self.0 & rhs)
                }
            }
            impl core::ops::AddAssign<usize> for $name {
                fn add_assign(&mut self, rhs: usize) {
                    self.0 += rhs
                }
            }
            impl core::ops::SubAssign<usize> for $name {
                fn sub_assign(&mut self, rhs: usize) {
                    self.0 -= rhs
                }
            }
        )*
    };
}

impl VirtAddr {
    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as _
    }
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as _
    }
}

impl VirtRegion {
    pub fn alloc_rootserver_obj(&mut self, size_bits: usize, num: usize) -> VirtAddr {
        let allocated = self.start;
        self.start += num * bit!(size_bits);
        assert!(allocated.raw() % bit!(size_bits) == 0);
        assert!(self.start <= self.end);

        unsafe {
            core::ptr::write_bytes(allocated.as_mut_ptr::<u8>(), 0, bit!(size_bits));
        }
        return allocated;
    }
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
