use crate::{
    arch::VirtRegion,
    boot::{consts::RootCNodeCapSlots, root_server::RootServerMem},
    object::{
        cspace::CNode,
        structures::{PageTableCap, VSpaceCap},
    },
};

use super::{IT_ASID, NUM_CONTEXT_REGS};

pub struct ArchTCB {
    context: UserContext,
}

pub struct UserContext {
    regs: [usize; NUM_CONTEXT_REGS],
}

impl CNode {
    pub fn create_it_address_space(
        &mut self,
        root_server_mem: &mut RootServerMem,
        it_v_reg: VirtRegion,
    ) {
        let cap = VSpaceCap::empty()
            .with_vs_mapped_asid(IT_ASID)
            .with_vs_base_ptr(root_server_mem.vspace.vspace_addr())
            .with_vs_is_mapped(true);

        self.write(RootCNodeCapSlots::InitThreadVSpace as _, cap);
        for addr in
            (it_v_reg.start.align_down(39).raw()..it_v_reg.end.align_up(39).raw()).step_by(bit!(39))
        {
            let page_ptr = root_server_mem.alloc_paging();
            let cap = PageTableCap::empty()
                .with_pt_base_ptr(page_ptr)
                .with_pt_is_mapped(true)
                .with_pt_mapped_asid(IT_ASID)
                .with_pt_mapped_address(addr);
            log::debug!("map cap: {:#x?}", cap);
        }

        for addr in
            (it_v_reg.start.align_down(30).raw()..it_v_reg.end.align_up(30).raw()).step_by(bit!(30))
        {
        }

        for addr in
            (it_v_reg.start.align_down(21).raw()..it_v_reg.end.align_up(21).raw()).step_by(bit!(21))
        {
        }
    }
}
