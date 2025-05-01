use super::PPTR_BASE;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PhysAddr(usize);
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtAddr(usize);

pub struct VirtAddrRange {
    pub start: VirtAddr,
    pub end: VirtAddr,
}

pub struct PhysAddrRange {
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
        )*
    };
}

impl_addr!(PhysAddr, VirtAddr);

impl PhysAddr {
    pub const fn vaddr(&self) -> VirtAddr {
        VirtAddr(self.0 | PPTR_BASE)
    }
}

macro_rules! pa {
    ($addr:expr) => {
        $crate::arch::PhysAddr::new($addr as usize)
    };
}

macro_rules! va {
    ($addr:expr) => {
        VirtAddr::new($addr as usize)
    };
}
