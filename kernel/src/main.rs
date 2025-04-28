#![no_std]
#![no_main]
#![feature(decl_macro)]

#[macro_use]
mod console;

mod arch;
mod driver;
mod lang_items;

use driver::system_off;

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
    console::init();
    log::debug!("phys_start: {:#x}", _phys_start);
    log::debug!("phys_end: {:#x}", _phys_end);
    log::debug!("phys_to_virt_offset: {:#x}", _phys_to_virt_offset);
    log::debug!("virt_entry: {:#x}", _virt_entry);
    log::debug!("dtb_addr_p: {:#x}", _dtb_addr_p);
    log::debug!("dtb_size: {:#x}", _dtb_size);
    system_off();
}
