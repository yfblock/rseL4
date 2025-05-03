use super::consts::{RootCNodeCapSlots, BOOT_INFO_FRAME_BITS};
use crate::{
    arch::{
        arch_get_n_paging, KAddr, KVirtRegion, VirtRegion, ASID_POOL_BITS, PAGE_BITS, SLOT_BITS,
        TCB_BITS, VSPACE_BITS, WORD_BITS,
    },
    config::ROOT_CNODE_SIZE_BITS,
    object::{cspace::CNode, structures::CNodeCap},
};

#[derive(Default)]
pub struct RootServerMem {
    pub cnode: CNode,
    pub vspace: KAddr,
    pub asid_pool: KAddr,
    pub ipc_buf: KAddr,
    pub boot_info: KAddr,
    pub extra_bi: KAddr,
    pub tcb: KAddr,
    pub paging: KVirtRegion,
}

impl RootServerMem {
    pub fn create_root_cnode(&mut self) {
        let cap = CNodeCap::empty()
            .with_cnode_radix(ROOT_CNODE_SIZE_BITS)
            .with_cnode_guard_size(WORD_BITS - ROOT_CNODE_SIZE_BITS)
            .with_cnode_ptr(self.cnode.cnode_addr().raw());
        self.cnode
            .write(RootCNodeCapSlots::InitThreadCNode as _, cap);
    }

    pub fn alloc_paging(&mut self) -> KAddr {
        let allocated = self.paging.start;
        self.paging.start += bit!(PAGE_BITS);
        assert!(self.paging.start <= self.paging.end);
        return allocated;
    }
}

pub fn calculate_rootserver_size(it_v_reg: VirtRegion, extra_bi_size_bits: usize) -> usize {
    let mut size = bit!(ROOT_CNODE_SIZE_BITS + SLOT_BITS)
        + bit!(TCB_BITS)
        + bit!(PAGE_BITS)
        + bit!(BOOT_INFO_FRAME_BITS)
        + bit!(ASID_POOL_BITS)
        + bit!(VSPACE_BITS);
    if extra_bi_size_bits > 0 {
        size += bit!(extra_bi_size_bits);
    }
    return size + arch_get_n_paging(it_v_reg) * bit!(PAGE_BITS);
}

#[inline]
pub fn rootserver_max_size_bits(extra_bi_size_bits: usize) -> usize {
    let cnode_size_bits = ROOT_CNODE_SIZE_BITS + SLOT_BITS;
    let max = cnode_size_bits.max(VSPACE_BITS);
    max.max(extra_bi_size_bits)
}

pub fn create_rootserver_objects(
    start: KAddr,
    it_v_reg: VirtRegion,
    extra_bi_size_bits: usize,
) -> RootServerMem {
    let cnode_size_bits = ROOT_CNODE_SIZE_BITS + SLOT_BITS;
    let max = rootserver_max_size_bits(extra_bi_size_bits);

    let size = calculate_rootserver_size(it_v_reg, extra_bi_size_bits);
    let mut rootserver = RootServerMem::default();
    let mut virt_region = KVirtRegion::new(start, start + size);

    if extra_bi_size_bits >= max && rootserver.extra_bi.is_null() {
        rootserver.extra_bi = virt_region.alloc_rootserver_obj(extra_bi_size_bits, 1);
    }

    // 申请 CSpace 和 VSpace
    rootserver.cnode = CNode::new(virt_region.alloc_rootserver_obj(cnode_size_bits, 1));
    if extra_bi_size_bits >= VSPACE_BITS && rootserver.extra_bi.is_null() {
        rootserver.extra_bi = virt_region.alloc_rootserver_obj(extra_bi_size_bits, 1);
    }
    rootserver.vspace = virt_region.alloc_rootserver_obj(VSPACE_BITS, 1);

    // 申请 asid_poll 和 ipc_buf
    if extra_bi_size_bits >= PAGE_BITS && rootserver.extra_bi.is_null() {
        rootserver.extra_bi = virt_region.alloc_rootserver_obj(extra_bi_size_bits, 1);
    }
    rootserver.asid_pool = virt_region.alloc_rootserver_obj(ASID_POOL_BITS, 1);
    rootserver.ipc_buf = virt_region.alloc_rootserver_obj(PAGE_BITS, 1);

    // 申请存储 BootInfo 需要的内存
    rootserver.boot_info = virt_region.alloc_rootserver_obj(BOOT_INFO_FRAME_BITS, 1);

    // 申请映射 Initial Thread 构建页表需要的内存
    let n = arch_get_n_paging(it_v_reg);
    rootserver.paging.start = virt_region.alloc_rootserver_obj(PAGE_BITS, n);
    rootserver.paging.end = rootserver.paging.start + n * bit!(PAGE_BITS);

    // 申请 TCB
    rootserver.tcb = virt_region.alloc_rootserver_obj(TCB_BITS, 1);

    assert!(virt_region.start == virt_region.end);
    return rootserver;
}
