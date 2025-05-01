use core::arch::global_asm;

global_asm!(include_defines!(), include_str!("trap.S"));

#[no_mangle]
unsafe extern "C" fn c_handle_interrupt() {}

#[no_mangle]
unsafe extern "C" fn c_handle_data_fault() {}

#[no_mangle]
unsafe extern "C" fn c_handle_instruction_fault() {}

#[no_mangle]
unsafe extern "C" fn c_handle_syscall() {}

#[no_mangle]
unsafe extern "C" fn c_handle_undefined_instruction() {}
