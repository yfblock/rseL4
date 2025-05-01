use core::panic::PanicInfo;

use crate::driver::system_off;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "\x1b[1;31m [{}:{}]\x1b[0m",
            location.file(),
            location.line(),
        );
    }
    println!("\x1b[1;31m panic: '{}'\x1b[0m", info.message());
    system_off()
}
