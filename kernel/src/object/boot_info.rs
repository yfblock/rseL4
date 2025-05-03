use crate::{
    arch::VirtAddr,
    boot::{
        consts::{RootCNodeCapSlots, BOOT_INFO_FRAME_BITS},
        root_server::RootServerMem,
        BootInfo, NDKS_BOOT,
    },
    config::ROOT_CNODE_SIZE_BITS,
    state_data::{KS_DOM_SCHEDULE, KS_DOM_SCHEDULE_IDX},
};

pub struct BootInfoHeader {
    id: usize,
    len: usize,
}

pub fn populate_bi_frame(
    root_server_mem: &mut RootServerMem,
    node_id: usize,
    num_nodes: usize,
    ipcbuf_vptr: VirtAddr,
    extra_bi_size: usize,
) {
    unsafe {
        core::ptr::write_bytes(
            root_server_mem.boot_info.as_mut_ptr::<u8>(),
            0,
            bit!(BOOT_INFO_FRAME_BITS),
        );
    }

    if extra_bi_size > 0 {
        // TODO: 初始化 extra_bi 的内存
    }

    let bi = unsafe {
        root_server_mem
            .boot_info
            .as_mut_ptr::<BootInfo>()
            .as_mut()
            .expect("invalid boot_info pointer")
    };
    bi.node_id = node_id;
    bi.num_nodes = num_nodes;
    bi.num_io_pt_levels = 0;
    bi.ipc_buffer = ipcbuf_vptr;
    bi.it_cnode_size_bits = ROOT_CNODE_SIZE_BITS;
    bi.it_domain = KS_DOM_SCHEDULE.lock()[*KS_DOM_SCHEDULE_IDX.lock()].domain;
    bi.extra_len = extra_bi_size;

    let ndks_boot = NDKS_BOOT.check_lock();
    log::debug!(
        "root_server_mem.boot_info: {:#x}",
        root_server_mem.boot_info
    );
    log::debug!("ndks_boot: {:p}", ndks_boot);
    log::debug!("bi_frame: {:#p}", bi as *mut BootInfo);
    ndks_boot.bi_frame = va!(bi as *mut BootInfo);
    log::debug!("ndks_boot.bi_frame: {:#x?}", ndks_boot.bi_frame);
    ndks_boot.slot_pos_cur = RootCNodeCapSlots::NumInitialCaps as _;
}
