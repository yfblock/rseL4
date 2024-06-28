use core::panic::PanicInfo;

use polyhal::{hart_id, shutdown};

use crate::console::println;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "\x1b[1;31m[Core {}] [{}:{}]\x1b[0m",
            hart_id(),
            location.file(),
            location.line(),
        );
    }
    println!(
        "\x1b[1;31m[Core {}] panic: '{}'\x1b[0m",
        hart_id(),
        info.message().unwrap()
    );
    // backtrace();
    println!("!TEST FINISH!");
    // loop {}
    shutdown()
}
