use core::ops::Range;

use crate::{
    arch::{KVirtRegion, VAddr, VirtRegion},
    boot::{consts::RootCNodeCapSlots, root_server::RootServerMem, BootInfo, NDKS_BOOT},
    object::{
        cap::CapTrait,
        cspace::CNode,
        structures::{AsidControlCap, AsidPoolCap, FrameCap, PageTableCap, VSpaceCap},
    },
};

use super::{VSpace, ASID_POOL_INDEX_BITS, IT_ASID, NUM_CONTEXT_REGS, PAGE_BITS};

pub struct ArchTCB {
    context: UserContext,
}

pub struct UserContext {
    regs: [usize; NUM_CONTEXT_REGS],
}

impl CNode {
    fn provide_cap(&mut self, cap: impl CapTrait) {
        let ndks_boot = NDKS_BOOT.check_lock();
        self.write(ndks_boot.slot_pos_cur, &cap);
        ndks_boot.slot_pos_cur += 1;
    }

    pub fn create_it_address_space(
        &mut self,
        root_server_mem: &mut RootServerMem,
        it_v_reg: VirtRegion,
    ) -> VSpaceCap {
        let slot_pos_before = NDKS_BOOT.slot_pos_cur;
        let cap = VSpaceCap::empty()
            .with_vs_mapped_asid(IT_ASID)
            .with_vs_base_ptr(root_server_mem.vspace.vspace_addr())
            .with_vs_is_mapped(true);
        self.write(RootCNodeCapSlots::InitThreadVSpace as _, &cap);

        for addr in
            (it_v_reg.start.align_down(39).raw()..it_v_reg.end.align_up(39).raw()).step_by(bit!(39))
        {
            let page_ptr = root_server_mem.alloc_paging();
            let cap = PageTableCap::empty()
                .with_pt_base_ptr(page_ptr)
                .with_pt_is_mapped(true)
                .with_pt_mapped_asid(IT_ASID)
                .with_pt_mapped_address(addr);
            root_server_mem.vspace.map_it_pud(&cap);
            self.provide_cap(cap);
        }

        for addr in
            (it_v_reg.start.align_down(30).raw()..it_v_reg.end.align_up(30).raw()).step_by(bit!(30))
        {
            let page_ptr = root_server_mem.alloc_paging();
            let cap = PageTableCap::empty()
                .with_pt_base_ptr(page_ptr)
                .with_pt_is_mapped(true)
                .with_pt_mapped_asid(IT_ASID)
                .with_pt_mapped_address(addr);
            root_server_mem.vspace.map_it_pd(&cap);
            self.provide_cap(cap);
        }

        for addr in
            (it_v_reg.start.align_down(21).raw()..it_v_reg.end.align_up(21).raw()).step_by(bit!(21))
        {
            let page_ptr = root_server_mem.alloc_paging();
            let cap = PageTableCap::empty()
                .with_pt_base_ptr(page_ptr)
                .with_pt_is_mapped(true)
                .with_pt_mapped_asid(IT_ASID)
                .with_pt_mapped_address(addr);
            root_server_mem.vspace.map_it_pt(&cap);
            self.provide_cap(cap);
        }
        let slot_pos_after = NDKS_BOOT.slot_pos_cur;

        NDKS_BOOT.bi_frame.get_mut::<BootInfo>().user_image_paging =
            slot_pos_before..slot_pos_after;
        cap
    }

    pub fn create_bi_frame_cap(
        &mut self,
        root_server_mem: &mut RootServerMem,
        vspace: &VSpaceCap,
        vptr: VAddr,
    ) {
        let mut vspace = VSpace::new(vspace.get_vs_base_ptr());
        let cap = vspace.create_mapped_it_frame_cap(
            root_server_mem.boot_info,
            vptr,
            IT_ASID,
            false,
            false,
        );
        self.write(RootCNodeCapSlots::BootInfoFrame as _, &cap);
    }

    pub fn create_ipcbuf_frame_cap(
        &mut self,
        root_server_mem: &mut RootServerMem,
        vspace: &VSpaceCap,
        vptr: VAddr,
    ) -> FrameCap {
        let mut vspace = VSpace::new(vspace.get_vs_base_ptr());
        let cap =
            vspace.create_mapped_it_frame_cap(root_server_mem.ipc_buf, vptr, IT_ASID, false, false);
        self.write(RootCNodeCapSlots::InitThreadIPCBuffer as _, &cap);
        cap
    }

    pub fn create_frames_of_region(
        &mut self,
        vspace: &VSpaceCap,
        mut reg: KVirtRegion,
        do_map: bool,
        pv_offset: usize,
    ) -> Result<Range<usize>, ()> {
        let mut vspace = VSpace::new(vspace.get_vs_base_ptr());
        let slot_pos_before = NDKS_BOOT.slot_pos_cur;

        while reg.start < reg.end {
            let frame_cap = if do_map {
                vspace.create_mapped_it_frame_cap(
                    reg.start,
                    va!((reg.start - pv_offset).paddr().raw()),
                    IT_ASID,
                    false,
                    true,
                )
            } else {
                todo!("create_unmapped_it_frame_cap")
            };

            self.provide_cap(frame_cap);

            reg.start += bit!(PAGE_BITS);
        }

        let slot_pos_after = NDKS_BOOT.slot_pos_cur;

        Ok(slot_pos_before..slot_pos_after)
    }

    pub fn create_it_asid_pool(&mut self, root_server_mem: &mut RootServerMem) -> AsidPoolCap {
        let cap = AsidPoolCap::empty()
            .with_asid_base(IT_ASID >> ASID_POOL_INDEX_BITS)
            .with_asid_pool(root_server_mem.asid_pool.raw());
        self.write(RootCNodeCapSlots::InitThreadASIDPool as _, &cap);
        self.write(
            RootCNodeCapSlots::ASIDControl as _,
            &AsidControlCap::empty(),
        );
        cap
    }
}
