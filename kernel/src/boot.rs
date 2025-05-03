use crate::{
    arch::{
        arch_get_n_paging, PhysRegion, VirtAddr, VirtRegion, ASID_POOL_BITS, PAGE_BITS, PPTR_TOP,
        SLOT_BITS, TCB_BITS, VSPACE_BITS,
    },
    config::ROOT_CNODE_SIZE_BITS,
};
use arrayvec::ArrayVec;
use core::ptr::NonNull;

/// TODO: use dynamic 设置
const MAX_NUM_RESV_REG: usize = 10;
/// TODO: use dynamic 设置
const MAX_NUM_FREEMEM_REG: usize = 10;
pub const BOOT_INFO_FRAME_BITS: usize = 12;

#[derive(Default)]
pub struct RootServerMem {
    cnode: VirtAddr,
    vspace: VirtAddr,
    asid_pool: VirtAddr,
    ipc_buf: VirtAddr,
    boot_info: VirtAddr,
    extra_bi: VirtAddr,
    tcb: VirtAddr,
    paging: VirtRegion,
}

// pub struct BootInfo {
//     seL4_Word         extraLen;        /* length of any additional bootinfo information */
//     seL4_NodeId       nodeID;          /* ID [0..numNodes-1] of the seL4 node (0 if uniprocessor) */
//     seL4_Word         numNodes;        /* number of seL4 nodes (1 if uniprocessor) */
//     seL4_Word         numIOPTLevels;   /* number of IOMMU PT levels (0 if no IOMMU support) */
//     seL4_IPCBuffer   *ipcBuffer;       /* pointer to initial thread's IPC buffer */
//     seL4_SlotRegion   empty;           /* empty slots (null caps) */
//     seL4_SlotRegion   sharedFrames;    /* shared-frame caps (shared between seL4 nodes) */
//     seL4_SlotRegion   userImageFrames; /* userland-image frame caps */
//     seL4_SlotRegion   userImagePaging; /* userland-image paging structure caps */
//     seL4_SlotRegion   ioSpaceCaps;     /* IOSpace caps for ARM SMMU */
//     seL4_SlotRegion   extraBIPages;    /* caps for any pages used to back the additional bootinfo information */
//     seL4_Word         initThreadCNodeSizeBits; /* initial thread's root CNode size (2^n slots) */
//     seL4_Domain       initThreadDomain; /* Initial thread's domain ID */
// #ifdef CONFIG_KERNEL_MCS
//     seL4_SlotRegion   schedcontrol; /* Caps to sched_control for each node */
// #endif
//     seL4_SlotRegion   untyped;         /* untyped-object caps (untyped caps) */
//     seL4_UntypedDesc  untypedList[CONFIG_MAX_NUM_BOOTINFO_UNTYPED_CAPS]; /* information about each untyped */
//     /* the untypedList should be the last entry in this struct, in order
//      * to make this struct easier to represent in other languages */
// } seL4_BootInfo;
pub struct BootInfo {}

pub struct NdksBoot {
    reserved: ArrayVec<PhysRegion, MAX_NUM_RESV_REG>,
    reserved_count: usize,
    freemem: ArrayVec<VirtRegion, MAX_NUM_FREEMEM_REG>,
    bi_frame: NonNull<BootInfo>,
    slot_pos_cur: usize,
}

impl NdksBoot {
    pub const fn empty() -> Self {
        NdksBoot {
            reserved: ArrayVec::new_const(),
            reserved_count: 0,
            freemem: ArrayVec::new_const(),
            bi_frame: NonNull::dangling(),
            slot_pos_cur: 0,
        }
    }

    pub fn reserve_region(&mut self, reg: PhysRegion) {
        assert!(reg.start < reg.end);
        if reg.start == reg.end {
            return;
        }
        for i in 0..self.reserved_count {
            let cur = self.reserved[i];

            // 合并前邻接区域
            if cur.start == reg.end {
                self.reserved[i].start = reg.start;
                self.merge_reserved_regions();
                return;
            }

            // 合并后邻接区域
            if cur.end == reg.start {
                self.reserved[i].end = reg.end;
                self.merge_reserved_regions();
                return;
            }

            // 插入到顺序中的正确位置
            if cur.start > reg.end {
                if self.reserved_count + 1 >= MAX_NUM_RESV_REG {
                    println!(
                        "Can't mark region {:#x?}-{:#x?} as reserved, increase MAX_NUM_RESV_REG ({})",
                        reg.start, reg.end, MAX_NUM_RESV_REG
                    );
                    return;
                }

                // 向后移动腾出空位
                for j in (i..self.reserved_count).rev() {
                    self.reserved[j + 1] = self.reserved[j];
                }

                self.reserved[i] = reg;
                self.reserved_count += 1;
                return;
            }
        }

        // 添加到末尾
        if self.reserved_count + 1 >= MAX_NUM_RESV_REG {
            println!(
                "Can't mark region {:#x?}-{:#x?} as reserved, increase MAX_NUM_RESV_REG ({})",
                reg.start, reg.end, MAX_NUM_RESV_REG
            );
            return;
        }

        self.reserved.push(reg);
        self.reserved_count += 1;
    }

    fn merge_reserved_regions(&mut self) {
        let mut i = 0;
        while i + 1 < self.reserved_count {
            let a = self.reserved[i];
            let b = self.reserved[i + 1];

            if a.end >= b.start {
                // 合并两个区域
                self.reserved[i].end = self.reserved[i].end.max(b.end);

                // 左移数组覆盖 b
                for j in i + 1..self.reserved_count - 1 {
                    self.reserved[j] = self.reserved[j + 1];
                }

                self.reserved_count -= 1;
                continue; // 检查 i 与新的 i+1
            }

            i += 1;
        }
    }
}

pub fn init_free_mem(
    availables: &[PhysRegion],
    reserveds: &[VirtRegion],
    it_v_reg: VirtRegion,
    extra_bi_size_bits: usize,
) -> RootServerMem {
    let mut ndks_boot = NdksBoot::empty();
    let mut avail_regs = ArrayVec::<VirtRegion, MAX_NUM_FREEMEM_REG>::new();
    avail_regs.extend(availables.iter().map(|x| x.pptr()));
    avail_regs.iter_mut().for_each(|x| {
        x.start = x.start.min(va!(PPTR_TOP));
        x.end = x.end.min(va!(PPTR_TOP));
    });
    // Rust 版本的 remove_reserved_regions 逻辑
    let mut a = 0;
    let mut r = 0;
    log::debug!("reserved region: {:#x?}", reserveds);

    while a < availables.len() && r < reserveds.len() {
        let reserved = reserveds[r];
        let avail = &mut avail_regs[a];

        if reserved.is_empty() {
            // 空保留区域
            r += 1;
        } else if avail.is_empty() {
            // 空可用区域
            a += 1;
        } else if reserved.end <= avail.start {
            // 保留区域在可用区域之前
            ndks_boot.reserve_region(reserved.paddr());
            r += 1;
        } else if reserved.start >= avail.end {
            // 保留区域在可用区域之后
            ndks_boot.freemem.push(*avail);
            a += 1;
        } else {
            // 有重叠
            if reserved.start <= avail.start {
                // 裁剪开头
                avail.start = avail.end.min(reserved.end);
                // 不前进 r，可能还重叠
            } else {
                // 分出不重叠前段
                ndks_boot
                    .freemem
                    .push(VirtRegion::new(avail.start, reserved.start));

                if avail.end > reserved.end {
                    // 裁剪后继续处理当前 avail
                    avail.start = reserved.end;
                } else {
                    // 完全被保留覆盖
                    a += 1;
                }
            }
        }
    }

    reserveds[r..]
        .iter()
        .for_each(|x| ndks_boot.reserve_region(x.paddr()));

    avail_regs[a..]
        .iter()
        .for_each(|x| ndks_boot.freemem.push(*x));

    log::debug!("a: {}  r: {}", a, r);
    log::debug!("ndks reserved: {:#x?}", &ndks_boot.reserved);
    log::debug!("ndks freemem : {:#x?}", &ndks_boot.freemem);

    // 确保留出一个空位
    assert!(ndks_boot.freemem.len() < ndks_boot.freemem.capacity());

    let size = calculate_rootserver_size(it_v_reg, extra_bi_size_bits);
    let max = rootserver_max_size_bits(extra_bi_size_bits);

    let mut i = ndks_boot.freemem.len() - 1;
    while i as isize >= 0 {
        let unaligned_start = ndks_boot.freemem[i].end - size;
        let start = unaligned_start.align_down(max);
        if unaligned_start <= ndks_boot.freemem[i].end && start >= ndks_boot.freemem[i].start {
            let root_server_mem = create_rootserver_objects(start, it_v_reg, extra_bi_size_bits);

            ndks_boot.freemem[i].end = start;
            ndks_boot
                .freemem
                .insert(i, VirtRegion::new(start + size, ndks_boot.freemem[i].end));
            return root_server_mem;
        }
        i -= 1;
    }
    panic!("No free memory region is big enough for root server");
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
fn rootserver_max_size_bits(extra_bi_size_bits: usize) -> usize {
    let cnode_size_bits = ROOT_CNODE_SIZE_BITS + SLOT_BITS;
    let max = cnode_size_bits.max(VSPACE_BITS);
    max.max(extra_bi_size_bits)
}

pub const fn get_n_paging(v_reg: VirtRegion, bits: usize) -> usize {
    let start = v_reg.start.align_down(bits).raw();
    let end = v_reg.end.align_up(bits).raw();
    (end - start) / bit!(bits)
}

fn create_rootserver_objects(
    start: VirtAddr,
    it_v_reg: VirtRegion,
    extra_bi_size_bits: usize,
) -> RootServerMem {
    let cnode_size_bits = ROOT_CNODE_SIZE_BITS + SLOT_BITS;
    let max = rootserver_max_size_bits(extra_bi_size_bits);

    let size = calculate_rootserver_size(it_v_reg, extra_bi_size_bits);
    let mut rootserver = RootServerMem::default();
    let mut virt_region = VirtRegion::new(start, start + size);

    if extra_bi_size_bits >= max && rootserver.extra_bi.is_null() {
        rootserver.extra_bi = virt_region.alloc_rootserver_obj(extra_bi_size_bits, 1);
    }

    // 申请 CSpace 和 VSpace
    rootserver.cnode = virt_region.alloc_rootserver_obj(cnode_size_bits, 1);
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
