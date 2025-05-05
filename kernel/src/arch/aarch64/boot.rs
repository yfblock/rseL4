use super::{mem::get_kernel_img_phys_region, NUM_RESERVED_REGIONS};
use crate::{
    arch::{
        aarch64::{
            cpu,
            gic::init_irqs,
            vspace::{map_kernel_window, AsidPool, KS_ASID_TABLE},
            PAGE_SIZE,
        },
        KVirtRegion, PAddr, PhysRegion, VirtRegion, ASID_POOL_INDEX_BITS, IT_ASID,
    },
    boot::{
        consts::BOOT_INFO_FRAME_BITS, init_free_mem, root_server::RootServerMem, BootInfo,
        NDKS_BOOT,
    },
    config::MAX_NUM_NODES,
    driver::system_off,
    object::{boot_info::populate_bi_frame, structures::AsidMapVSpace},
    platform::{PLAT_MEM_REGIONS, USER_TOP},
};
use core::arch::naked_asm;

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        include_defines!(),"
        msr     daifset, DAIFSET_MASK

        msr     spsel, #1
        mrs     x8, sctlr_el1
        // ldr     x19, =CR_BITS_SET
        // ldr     x20, =CR_BITS_CLEAR
        // orr     x8, x8, x19
        // bic     x8, x8, x20
        msr     sctlr_el1, x8

        ldr     x8, =kernel_boot_stack_top
        mov     sp, x8

        bl      {main}
        b       .",
        main = sym main
    );
}

/*
(kernel_entry)(
    payload_info.user_image.phys_addr_range.start,
    payload_info.user_image.phys_addr_range.end,
    0_usize.wrapping_sub(payload_info.user_image.phys_to_virt_offset) as isize,
    payload_info.user_image.virt_entry,
    dtb_addr_p,
    dtb_size,
)
*/
extern "C" fn main(
    ui_phys_start: PAddr,
    ui_phys_end: PAddr,
    pv_offset: usize,
    _virt_entry: usize,
    dtb_p_addr: PAddr,
    dtb_size: usize,
) -> ! {
    map_kernel_window();
    cpu::init_cpu();
    cpu::init_plat();
    crate::console::init();
    log::debug!("phys_start: {:#x?}", ui_phys_start);
    log::debug!("phys_end  : {:#x?}", ui_phys_end);
    log::debug!("pv_offset : {:#x?}", pv_offset);
    log::debug!("virt_entry: {:#x?}", _virt_entry);
    log::debug!("dtb_p_addr: {:#x?}", dtb_p_addr);
    log::debug!("dtb_size  : {:#x?}", dtb_size);

    let ui_p_reg = PhysRegion::new(ui_phys_start, ui_phys_end);
    let ui_reg = ui_p_reg.pptr();
    let ui_v_reg = VirtRegion::new(
        va!(ui_phys_start.raw() - pv_offset),
        va!(ui_phys_end.raw() - pv_offset),
    );
    let ipc_buffer_ptr = ui_v_reg.end;
    let bi_frame_vptr = ipc_buffer_ptr + PAGE_SIZE;
    let extra_bi_frame_vptr = bi_frame_vptr + bit!(BOOT_INFO_FRAME_BITS);
    log::debug!("ui_v_reg  : {:#x?}", ui_v_reg);
    log::debug!("ui_reg    ： {:#x?}", ui_reg);

    // let mut dtb_p_reg = PhysAddrRange::empty();
    // if dtb_size > 0 {
    //     let dtb_phys_end: PhysAddr = dtb_phys_addr + dtb_size;
    //     if dtb_phys_end < dtb_phys_addr {
    //         panic!(
    //             "ERROR: DTB location at {:#x?} len {:#x} invalid",
    //             dtb_phys_addr, dtb_size
    //         );
    //     }
    //     if dtb_phys_end.raw() > PADDR_TOP {
    //         panic!(
    //             "ERROR: DTB at [{:#x}..{:#x}] exceeds PADDR_TOP ({:#x?})",
    //             dtb_phys_addr.raw(),
    //             dtb_phys_end.raw(),
    //             PADDR_TOP
    //         );
    //     }
    // }

    // TODO: 添加设备树相关的信息
    // TODO: 为 BootInfo 添加额外的信息
    let extra_bi_size_bits = 0;
    let it_v_reg = VirtRegion::new(ui_v_reg.start, extra_bi_frame_vptr);

    if it_v_reg.end >= USER_TOP {
        panic!(
            "ERROR: userland image virt [{:#x}..{:#x}] exceeds USER_TOP ({:#x})",
            it_v_reg.start.raw(),
            it_v_reg.end.raw(),
            USER_TOP.raw()
        );
    }
    let mut root_server_mem = arch_init_free_mem(ui_p_reg, it_v_reg, extra_bi_size_bits);
    root_server_mem.create_root_cnode();
    root_server_mem.cnode.create_domain_cap();

    init_irqs(&mut root_server_mem.cnode);

    // TODO: Add Extra BI Size
    populate_bi_frame(&mut root_server_mem, 0, MAX_NUM_NODES, ipc_buffer_ptr, 0);

    // TODO: 检查 DTB 大小并修改 EXTRA_BIT，修改 header
    NDKS_BOOT.bi_frame.get_mut::<BootInfo>().io_space_caps = 0..0;

    let mut root_cnode = root_server_mem.cnode.clone();
    let vspace_cap = root_cnode.create_it_address_space(&mut root_server_mem, it_v_reg);

    root_cnode.create_bi_frame_cap(&mut root_server_mem, &vspace_cap, bi_frame_vptr);

    let ipcbuf_cap =
        root_cnode.create_ipcbuf_frame_cap(&mut root_server_mem, &vspace_cap, ipc_buffer_ptr);

    let create_frames_ret =
        root_cnode.create_frames_of_region(&vspace_cap, ui_reg, true, pv_offset);

    NDKS_BOOT.bi_frame.get_mut::<BootInfo>().user_image_frames = create_frames_ret.unwrap();

    let it_ap_cap = root_cnode.create_it_asid_pool(&mut root_server_mem);
    // it_ap_cap.get_asid_base()
    let asid_map_vspace =
        AsidMapVSpace::empty().with_vspace_root(vspace_cap.get_vs_base_ptr().raw());
    log::debug!("cap asid base: {:#x}", it_ap_cap.get_asid_pool());
    todo!("field_high implementation of get asid pool");
    AsidPool::from_cap(it_ap_cap).get_pool()[IT_ASID] = asid_map_vspace;
    KS_ASID_TABLE.lock()[IT_ASID >> ASID_POOL_INDEX_BITS] = ka!(it_ap_cap.get_asid_pool());
    todo!("write it asid pool");
    log::debug!("created bi frame");
    system_off();
}

pub fn arch_init_free_mem(
    ui_p_reg: PhysRegion,
    it_v_region: VirtRegion,
    extra_bi_size_bits: usize,
) -> RootServerMem {
    let mut reserved = [KVirtRegion::empty(); NUM_RESERVED_REGIONS];
    /* reserve the kernel image region */
    reserved[0] = get_kernel_img_phys_region().pptr();

    let mut index = 1;

    let ui_reg = ui_p_reg.pptr();
    reserved[index] = ui_reg;
    index += 1;

    init_free_mem(
        PLAT_MEM_REGIONS,
        &reserved[..index],
        it_v_region,
        extra_bi_size_bits,
    )
}
