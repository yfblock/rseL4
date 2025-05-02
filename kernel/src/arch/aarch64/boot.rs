use super::{mem::get_kernel_img_phys_region, NUM_RESERVED_REGIONS};
use crate::{
    arch::{
        aarch64::{cpu, vspace::map_kernel_window, PAGE_SIZE},
        PhysAddr, PhysRegion, VirtRegion,
    },
    boot::{init_free_mem, BOOT_INFO_FRAME_BITS},
    driver::system_off,
    platform::{PLAT_MEM_REGIONS, USER_TOP},
};
use core::arch::naked_asm;
use spin::Once;

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
    ui_phys_start: PhysAddr,
    ui_phys_end: PhysAddr,
    pv_offset: usize,
    _virt_entry: usize,
    dtb_p_addr: PhysAddr,
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
    let ui_v_reg = VirtRegion::new(
        va!(ui_phys_start.raw() - pv_offset),
        va!(ui_phys_end.raw() - pv_offset),
    );
    let ipc_buffer_ptr = ui_v_reg.end;
    let bi_frame_vptr = ipc_buffer_ptr + PAGE_SIZE;
    let extra_bi_frame_vptr = bi_frame_vptr + bit!(BOOT_INFO_FRAME_BITS);
    log::debug!("ui_v_reg  : {:#x?}", ui_v_reg);

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
    let it_v_reg = VirtRegion::new(ui_v_reg.start, extra_bi_frame_vptr);

    if it_v_reg.end >= USER_TOP {
        panic!(
            "ERROR: userland image virt [{:#x}..{:#x}] exceeds USER_TOP ({:#x})",
            it_v_reg.start.raw(),
            it_v_reg.end.raw(),
            USER_TOP.raw()
        );
    }
    arch_init_free_mem(ui_p_reg, it_v_reg);
    system_off();
}

static mut RESERVED: Once<[VirtRegion; NUM_RESERVED_REGIONS]> = Once::new();

pub fn arch_init_free_mem(ui_p_reg: PhysRegion, it_v_region: VirtRegion) {
    let mut reserved = [VirtRegion::empty(); NUM_RESERVED_REGIONS];
    /* reserve the kernel image region */
    reserved[0] = get_kernel_img_phys_region().pptr();

    let mut index = 1;

    let ui_reg = ui_p_reg.pptr();
    reserved[index] = ui_reg;
    index += 1;

    init_free_mem(PLAT_MEM_REGIONS, &reserved[..index], it_v_region);

    // /* avail_p_regs comes from the auto-generated code */
    // return init_freemem(ARRAY_SIZE(avail_p_regs), avail_p_regs,
    //                     index, reserved,
    //                     it_v_reg, extra_bi_size_bits);
}
