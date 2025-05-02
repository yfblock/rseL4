use crate::{
    arch::{PhysRegion, VirtRegion},
    boot::get_n_paging,
    platform::PADDR_ELF_ADDR,
};

use super::{PAGE_BITS, PAGE_TABLE_INDEX_BITS};

pub fn get_kernel_img_phys_region() -> PhysRegion {
    extern "C" {
        fn _ekernel();
    }
    PhysRegion {
        start: pa!(PADDR_ELF_ADDR),
        end: pa!(_ekernel),
    }
}

pub const fn arch_get_n_paging(it_v_reg: VirtRegion) -> usize {
    get_n_paging(it_v_reg, 3 * PAGE_TABLE_INDEX_BITS + PAGE_BITS)
        + get_n_paging(it_v_reg, 2 * PAGE_TABLE_INDEX_BITS + PAGE_BITS)
        + get_n_paging(it_v_reg, PAGE_TABLE_INDEX_BITS + PAGE_BITS)
}
