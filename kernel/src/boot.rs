use core::ptr::NonNull;

use crate::{
    arch::{
        arch_get_n_paging, PhysRegion, VirtRegion, ASID_POOL_BITS, PAGE_BITS, PPTR_TOP, SLOT_BITS,
        TCB_BITS, VSPACE_BITS,
    },
    config::ROOT_CNODE_SIZE_BITS,
};

/// TODO: use dynamic 设置
const MAX_NUM_RESV_REG: usize = 10;
/// TODO: use dynamic 设置
const MAX_NUM_FREEMEM_REG: usize = 10;

pub const BOOT_INFO_FRAME_BITS: usize = 12;

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
    reserved: [PhysRegion; MAX_NUM_RESV_REG],
    reserved_count: usize,
    freemem: [VirtRegion; MAX_NUM_FREEMEM_REG],
    bi_frarme: NonNull<BootInfo>,
    slot_pos_cur: usize,
}

impl NdksBoot {
    pub fn insert_region(&mut self, region: VirtRegion) {
        self.freemem
            .iter_mut()
            .find(|x| x.is_empty())
            .map(|x| *x = region);
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

        self.reserved[self.reserved_count] = reg;
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

pub fn init_free_mem(availables: &[PhysRegion], reserveds: &[VirtRegion], it_v_reg: VirtRegion) {
    let mut ndks_boot = NdksBoot {
        reserved: [PhysRegion::empty(); MAX_NUM_RESV_REG],
        reserved_count: 0,
        freemem: [VirtRegion::empty(); MAX_NUM_FREEMEM_REG],
        bi_frarme: NonNull::dangling(),
        slot_pos_cur: 0,
    };
    let mut avail_regs = [VirtRegion::empty(); MAX_NUM_FREEMEM_REG];
    for (region, avail_reg) in availables.iter().zip(avail_regs.iter_mut()) {
        *avail_reg = region.pptr();
        avail_reg.start = avail_reg.start.min(va!(PPTR_TOP));
        avail_reg.end = avail_reg.end.min(va!(PPTR_TOP));
    }
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
            ndks_boot.insert_region(*avail);
            a += 1;
        } else {
            // 有重叠
            if reserved.start <= avail.start {
                // 裁剪开头
                avail.start = avail.end.min(reserved.end);
                // 不前进 r，可能还重叠
            } else {
                // 分出不重叠前段
                let mut m = *avail;
                m.end = reserved.start;
                ndks_boot.insert_region(m);

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

    while r < reserveds.len() {
        ndks_boot.reserve_region(reserveds[r].paddr());
        r += 1;
    }

    while a < availables.len() {
        ndks_boot.insert_region(avail_regs[a]);
        a += 1;
    }

    log::debug!("a: {}  r: {}", a, r);
    log::debug!("ndks reserved: {:#x?}", &ndks_boot.reserved[..r]);
    log::debug!("ndks freemem : {:#x?}", &ndks_boot.freemem[..a]);

    // word_t a = 0;
    // word_t r = 0;
    // /* Now iterate through the available regions, removing any reserved regions. */
    // while (a < n_available && r < n_reserved) {
    //     if (reserved[r].start == reserved[r].end) {
    //         /* reserved region is empty - skip it */
    //         r++;
    //     } else if (avail_reg[a].start >= avail_reg[a].end) {
    //         /* skip the entire region - it's empty now after trimming */
    //         a++;
    //     } else if (reserved[r].end <= avail_reg[a].start) {
    //         /* the reserved region is below the available region - skip it */
    //         reserve_region(pptr_to_paddr_reg(reserved[r]));
    //         r++;
    //     } else if (reserved[r].start >= avail_reg[a].end) {
    //         /* the reserved region is above the available region - take the whole thing */
    //         insert_region(avail_reg[a]);
    //         a++;
    //     } else {
    //         /* the reserved region overlaps with the available region */
    //         if (reserved[r].start <= avail_reg[a].start) {
    //             /* the region overlaps with the start of the available region.
    //              * trim start of the available region */
    //             avail_reg[a].start = MIN(avail_reg[a].end, reserved[r].end);
    //             /* do not increment reserved index here - there could be more overlapping regions */
    //         } else {
    //             assert(reserved[r].start < avail_reg[a].end);
    //             /* take the first chunk of the available region and move
    //              * the start to the end of the reserved region */
    //             region_t m = avail_reg[a];
    //             m.end = reserved[r].start;
    //             insert_region(m);
    //             if (avail_reg[a].end > reserved[r].end) {
    //                 avail_reg[a].start = reserved[r].end;
    //                 /* we could increment reserved index here, but it's more consistent with the
    //                  * other overlapping case if we don't */
    //             } else {
    //                 a++;
    //             }
    //         }
    //     }
    // }
}

pub fn calculate_rootserver_size(it_v_reg: VirtRegion) -> usize {
    let size = bit!(ROOT_CNODE_SIZE_BITS + SLOT_BITS)
        + bit!(TCB_BITS)
        + bit!(PAGE_BITS)
        + bit!(BOOT_INFO_FRAME_BITS)
        + bit!(ASID_POOL_BITS)
        + bit!(VSPACE_BITS);
    return size + arch_get_n_paging(it_v_reg) * bit!(PAGE_BITS);
}

pub const fn get_n_paging(v_reg: VirtRegion, bits: usize) -> usize {
    let start = v_reg.start.raw() & (bit!(bits) - 1);
    let end = v_reg.end.raw().div_ceil(bit!(bits)) * bit!(bits);
    (end - start) / bit!(bits)
}
