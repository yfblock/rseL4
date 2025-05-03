use super::VSPACE_INDEX_BITS;
use crate::arch::PPTR_BASE;
use aarch64_cpu::{
    asm::barrier::{self, dsb},
    registers::{Writeable, TTBR0_EL1, TTBR1_EL1},
};
use hal::aarch64::{PTEFlags, PTE};

const PTE_LEN: usize = 512;

#[repr(align(4096))]
struct GlobalPageTable {
    /// 用户程序对应的设备地址
    user_vspace: [usize; bit!(VSPACE_INDEX_BITS)],
    /// PML4
    pgd: [PTE; PTE_LEN],
    /// 1GB Huge Page
    pud: [PTE; PTE_LEN],
    /// 2MB Large Page
    pds: [[PTE; PTE_LEN]; PTE_LEN],
    /// 内核设备对应的页表
    pt: [PTE; PTE_LEN],
}

impl GlobalPageTable {
    pub const fn new() -> Self {
        GlobalPageTable {
            user_vspace: [0; bit!(VSPACE_INDEX_BITS)],
            pgd: [PTE::empty(); PTE_LEN],
            pud: [PTE::empty(); PTE_LEN],
            pds: [[PTE::empty(); PTE_LEN]; PTE_LEN],
            pt: [PTE::empty(); PTE_LEN],
        }
    }
}

/// 刷新 TLB
#[inline]
pub fn flush_all() {
    unsafe { core::arch::asm!("tlbi vmalle1; dsb sy; isb") }
}

static mut GLOBAL_PT: GlobalPageTable = GlobalPageTable::new();

/// 映射内核内存
///
/// aarch64 为四级页表，在 [GLOBAL_PT] 映射内核内存，内存范围为 [PPTR_BASE] - [crate::arch::PPTR_TOP]，映射单位为 2MB 内存
pub fn map_kernel_window() {
    unsafe {
        let global_pt = (&raw mut GLOBAL_PT).as_mut().unwrap();
        global_pt.pgd[PTE_LEN - 1] = PTE::new_table(global_pt.pud.as_ptr() as usize - PPTR_BASE);
        for i in 0..PTE_LEN {
            global_pt.pud[i] = PTE::new_table(global_pt.pds[i].as_ptr() as usize - PPTR_BASE);
        }
        for i in 0..PTE_LEN * PTE_LEN {
            global_pt.pds[i / PTE_LEN][i % PTE_LEN] = PTE::new_page(
                i * 0x20_0000,
                PTEFlags::VALID | PTEFlags::AF | PTEFlags::ATTR_INDX | PTEFlags::NG,
            );
        }
    }
    // TODO: 映射设备物理内存并判断是否为用户态保留
}

/// 激活内核虚拟地址空间
///
/// 激活 [GLOBAL_PT] 中的虚拟地址空间，内核地址为 [GlobalPageTable::pgd]，用户地址空间为 [GlobalPageTable::user_vspace]
pub fn activate_kernel_vspace() {
    let (user_base_addr, kernel_base_addr) = unsafe {
        (
            (&raw mut GLOBAL_PT.user_vspace) as u64 - PPTR_BASE as u64,
            (&raw mut GLOBAL_PT.pgd) as u64 - PPTR_BASE as u64,
        )
    };
    dsb(barrier::SY);
    TTBR0_EL1.write(TTBR0_EL1::ASID.val(0));
    TTBR0_EL1.set_baddr(user_base_addr);

    TTBR1_EL1.write(TTBR1_EL1::ASID.val(0));
    TTBR1_EL1.set_baddr(kernel_base_addr);
    flush_all();
}
