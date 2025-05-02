use crate::{arch::PhysRegion, platform::PADDR_ELF_ADDR};

pub fn get_kernel_img_phys_region() -> PhysRegion {
    extern "C" {
        static _ekernel: usize;
    }
    unsafe {
        PhysRegion {
            start: pa!(PADDR_ELF_ADDR),
            end: pa!(_ekernel),
        }
    }
}
