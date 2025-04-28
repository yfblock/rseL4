use core::arch::naked_asm;

#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        "bl {main}",
        "b .",
        main = sym crate::main
    );
}
