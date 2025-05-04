use super::{VMRights, PAGE_BITS, VSPACE_INDEX_BITS};
use crate::{
    arch::{KAddr, VAddr, PPTR_BASE},
    object::structures::{FrameCap, PageTableCap},
};
use aarch64_cpu::{
    asm::barrier::{self, dsb},
    registers::{Writeable, TTBR0_EL1, TTBR1_EL1},
};
use hal::aarch64::{PTEFlags, PTE};

const PTE_LEN: usize = 512;
type PTEList = [PTE; PTE_LEN];

#[repr(align(4096))]
struct GlobalPageTable {
    /// 用户程序对应的设备地址
    user_vspace: [usize; bit!(VSPACE_INDEX_BITS)],
    /// PML4
    pgd: PTEList,
    /// 1GB Huge Page
    pud: PTEList,
    /// 2MB Large Page
    pds: [PTEList; PTE_LEN],
    /// 内核设备对应的页表
    pt: PTEList,
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

/// 获取地址对应的 level idex
const fn addr_level_index(addr: usize, level: usize) -> usize {
    (addr >> (PAGE_BITS + 9 * level)) & 0x1ff
}

static mut GLOBAL_PT: GlobalPageTable = GlobalPageTable::new();

/// 映射内核内存
///
/// aarch64 为四级页表，在 [GLOBAL_PT] 映射内核内存，内存范围为 [PPTR_BASE] - [crate::arch::PPTR_TOP]，映射单位为 2MB 内存
pub fn map_kernel_window() {
    let global_pt = ka!(&raw mut GLOBAL_PT).get_mut::<GlobalPageTable>();
    global_pt.pgd[PTE_LEN - 1] = PTE::new_table(global_pt.pud.as_ptr() as usize - PPTR_BASE);
    for i in 0..PTE_LEN {
        global_pt.pud[i] = PTE::new_table(global_pt.pds[i].as_ptr() as usize - PPTR_BASE);
    }
    for i in 0..PTE_LEN * PTE_LEN {
        global_pt.pds[i / PTE_LEN][i % PTE_LEN] = PTE::new_page(
            i * 0x20_0000,
            PTEFlags::V | PTEFlags::ACCESS | PTEFlags::ATTR_INDX | PTEFlags::NG,
        );
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

#[derive(Default, Clone)]
pub struct VSpace(KAddr);

impl VSpace {
    /// 创建一个 [VSpace] 结构体
    ///
    /// ## 参数
    /// - `addr` [VirtAddr] VSpace 指向的物理内存
    pub const fn new(addr: KAddr) -> Self {
        Self(addr)
    }

    /// 获取 [VSpace] 指向的内存地址 [VirtAddr]
    pub const fn vspace_addr(&self) -> KAddr {
        self.0
    }

    ///
    pub fn map_it_pud(&mut self, cap: &PageTableCap) {
        let map_addr = cap.get_pt_mapped_address();
        let pgd = self.vspace_addr().get_mut::<PTEList>();

        let pgd_index = addr_level_index(map_addr, 3);
        pgd[pgd_index] = PTE::new_table(cap.get_pt_base_ptr().paddr().raw());
    }

    pub fn map_it_pd(&mut self, cap: &PageTableCap) {
        let map_addr = cap.get_pt_mapped_address();
        let pgd = self.vspace_addr().get_mut::<PTEList>();

        let pgd_index = addr_level_index(map_addr, 3);
        let pud = pa!(pgd[pgd_index].addr()).pptr().get_mut::<PTEList>();

        let pud_index = addr_level_index(map_addr, 2);
        pud[pud_index] = PTE::new_table(cap.get_pt_base_ptr().paddr().raw());
    }

    pub fn map_it_pt(&mut self, cap: &PageTableCap) {
        let map_addr = cap.get_pt_mapped_address();
        let pgd = self.vspace_addr().get_mut::<PTEList>();

        let pgd_index = addr_level_index(map_addr, 3);
        let pud = pa!(pgd[pgd_index].addr()).pptr().get_mut::<PTEList>();

        let pud_index = addr_level_index(map_addr, 2);
        let pd = pa!(pud[pud_index].addr()).pptr().get_mut::<PTEList>();

        let pd_index = addr_level_index(map_addr, 1);
        pd[pd_index] = PTE::new_table(cap.get_pt_base_ptr().paddr().raw());
    }

    pub fn create_mapped_it_frame_cap(
        &mut self,
        pptr: KAddr,
        vptr: VAddr,
        asid: usize,
        use_large: bool,
        executable: bool,
    ) -> FrameCap {
        let frame_size = match use_large {
            true => bit!(20),
            false => bit!(12),
        };
        let cap = FrameCap::empty()
            .with_mapped_asid(asid)
            .with_base_ptr(pptr)
            .with_size(frame_size)
            .with_mapped_address(vptr.raw())
            .with_is_device(false)
            .with_fvm_rights(VMRights::VMReadWrite as _);
        let mut flags = PTEFlags::V | PTEFlags::ACCESS | PTEFlags::ATTR_INDX | PTEFlags::NG;

        if !executable {
            flags |= PTEFlags::PXN;
        }

        let map_addr = cap.get_mapped_address();
        let pgd = self.vspace_addr().get_mut::<PTEList>();

        let pgd_index = addr_level_index(map_addr, 3);
        let pud = pa!(pgd[pgd_index].addr()).pptr().get_mut::<PTEList>();

        let pud_index = addr_level_index(map_addr, 2);
        let pd = pa!(pud[pud_index].addr()).pptr().get_mut::<PTEList>();

        let pd_index = addr_level_index(map_addr, 1);
        let pt = pa!(pd[pd_index].addr()).pptr().get_mut::<PTEList>();

        let pt_index = addr_level_index(map_addr, 0);
        pt[pt_index] = PTE::new_page(cap.get_base_ptr().raw(), flags);

        return cap;
    }
}
