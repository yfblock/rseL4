pub mod boot_lock;
pub mod consts;
pub mod root_server;

use crate::{
    arch::{PhysRegion, VirtAddr, VirtRegion, PPTR_TOP},
    config::MAX_NUM_BOOTINFO_UNTYPED_CAPS,
};
use arrayvec::ArrayVec;
use boot_lock::BootLock;
use consts::{MAX_NUM_FREEMEM_REG, MAX_NUM_RESV_REG};
use core::ops::Range;
use root_server::{
    calculate_rootserver_size, create_rootserver_objects, rootserver_max_size_bits, RootServerMem,
};

pub static NDKS_BOOT: BootLock<NdksBoot> = BootLock::new(NdksBoot::empty());

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
#[repr(C)]
pub struct BootInfo {
    pub extra_len: usize,
    pub node_id: usize,
    pub num_nodes: usize,
    pub num_io_pt_levels: usize,
    pub ipc_buffer: VirtAddr,
    pub empty: Range<usize>,
    pub shared_frames: Range<usize>,
    pub user_image_frames: Range<usize>,
    pub user_image_paging: Range<usize>,
    pub io_space_caps: Range<usize>,
    pub extra_bi_pages: Range<usize>,
    pub it_cnode_size_bits: usize,
    pub it_domain: usize,
    pub untyped: Range<usize>,
    pub untyped_list: [UntypedDesc; MAX_NUM_BOOTINFO_UNTYPED_CAPS],
}

#[repr(C)]
pub struct UntypedDesc {
    paddr: usize,
    size_bits: u8,
    is_device: u8,
    _padding: u32,
}

pub struct NdksBoot {
    pub reserved: ArrayVec<PhysRegion, MAX_NUM_RESV_REG>,
    pub reserved_count: usize,
    pub freemem: ArrayVec<VirtRegion, MAX_NUM_FREEMEM_REG>,
    pub bi_frame: VirtAddr,
    pub slot_pos_cur: usize,
}

impl NdksBoot {
    pub const fn empty() -> Self {
        NdksBoot {
            reserved: ArrayVec::new_const(),
            reserved_count: 0,
            freemem: ArrayVec::new_const(),
            bi_frame: VirtAddr::new(0),
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
            *NDKS_BOOT.check_lock() = ndks_boot;
            return root_server_mem;
        }
        i -= 1;
    }
    panic!("No free memory region is big enough for root server");
}

pub const fn get_n_paging(v_reg: VirtRegion, bits: usize) -> usize {
    let start = v_reg.start.align_down(bits).raw();
    let end = v_reg.end.align_up(bits).raw();
    (end - start) / bit!(bits)
}
