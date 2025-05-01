use core::arch::naked_asm;

use crate::{
    arch::aarch64::{cpu, vspace::map_kernel_window},
    driver::system_off,
};

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
    _phys_start: usize,
    _phys_end: usize,
    _phys_to_virt_offset: isize,
    _virt_entry: usize,
    _dtb_addr_p: usize,
    _dtb_size: usize,
) -> ! {
    map_kernel_window();
    cpu::init_cpu();
    cpu::init_plat();
    crate::console::init();
    log::debug!("phys_start: {:#x}", _phys_start);
    log::debug!("phys_end: {:#x}", _phys_end);
    log::debug!("phys_to_virt_offset: {:#x}", _phys_to_virt_offset);
    log::debug!("virt_entry: {:#x}", _virt_entry);
    log::debug!("dtb_addr_p: {:#x}", _dtb_addr_p);
    log::debug!("dtb_size: {:#x}", _dtb_size);

    system_off();
}
