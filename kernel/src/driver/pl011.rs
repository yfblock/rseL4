//! PL011 UART.

use arm_pl011::Pl011Uart;
use spin::Mutex;

use crate::{arch::PhysAddr, console::Console};

const PL011_UART: PhysAddr = pa!(0x0900_0000);

static UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(PL011_UART.pptr().raw() as _));

impl Console {
    /// Writes a byte to the console.
    #[inline]
    pub fn putchar(c: u8) {
        match c {
            b'\n' => {
                UART.lock().putchar(b'\r');
                UART.lock().putchar(b'\n');
            }
            c => UART.lock().putchar(c),
        }
    }

    #[inline]
    pub fn init_uart() {
        UART.lock().init();
    }
}
