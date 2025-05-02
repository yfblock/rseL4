use core::ptr::NonNull;

use crate::{
    arch::{
        PhysRegion, VirtRegion, ASID_POOL_BITS, PAGE_BITS, PPTR_TOP, SLOT_BITS, TCB_BITS,
        VSPACE_BITS,
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

    pub fn reserved_region(&mut self, region: PhysRegion) {
        assert!(region.start < region.end);
        if region.start == region.end {
            return;
        }
        todo!("finish reserved region")
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
    for (i, region) in availables.iter().enumerate() {
        avail_regs[i] = region.pptr();
        if avail_regs[i].start.raw() >= PPTR_TOP {
            avail_regs[i].start = va!(PPTR_TOP);
        }
        if avail_regs[i].end.raw() >= PPTR_TOP {
            avail_regs[i].end = va!(PPTR_TOP);
        }
    }
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
    let mut size = bit!(ROOT_CNODE_SIZE_BITS + SLOT_BITS)
        + bit!(TCB_BITS)
        + bit!(PAGE_BITS)
        + bit!(BOOT_INFO_FRAME_BITS)
        + bit!(ASID_POOL_BITS)
        + bit!(VSPACE_BITS);
    // /* for all archs, seL4_PageTable Bits is the size of all non top-level paging structures */
    // return size + arch_get_n_paging(it_v_reg) * BIT(seL4_PageTableBits);
    todo!("calulate initialize thread virtual region")
}
