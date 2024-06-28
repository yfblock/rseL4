use core::fmt::Write;
use polyhal::debug::DebugConsole;

/// Print macro to print format without newline.
#[allow(unused_macros)]
pub(crate) macro print($fmt: expr $(, $($arg: tt)+)?) {
    $crate::console::_print(format_args!($fmt $(, $($arg)+)?))
}

/// Print macro to print format with newline
#[allow(unused_macros)]
pub(crate) macro println {
    () => {
        $crate::console::_print(format_args!("\n"))
    },
    ($fmt: expr $(, $($arg: tt)+)?) => {
        $crate::console::_print(format_args!("{}\n", format_args!($fmt $(, $($arg)+)?)))
    },
}

/// Print the given arguments
#[inline]
#[doc(hidden)]
pub(crate) fn _print(args: core::fmt::Arguments) {
    DebugConsole.write_fmt(args).expect("can't print arguments");
}
