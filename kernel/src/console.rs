use core::fmt::Write;

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes().iter().for_each(|&c| Console::putchar(c));
        Ok(())
    }
}

/// Print macro to print format without newline.
macro_rules! print {
    ($fmt:expr $(, $($arg: tt)+)?) => {
        $crate::console::_print(format_args!($fmt $(, $($arg)+)?))
    }
}

/// Print macro to print format with newline
macro_rules! println {
    () => {
        print!("\n")
    };
    ($fmt: expr $(, $($arg: tt)+)?) => {
        print!("{}\n", format_args!($fmt $(, $($arg)+)?))
    };
}

/// Print the given arguments
#[inline]
#[doc(hidden)]
pub(crate) fn _print(args: core::fmt::Arguments) {
    Console.write_fmt(args).unwrap();
}

impl log::Log for Console {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        use log::Level;
        let module = record.module_path();
        let line = record.line();
        // ANSI Color Code: https://i.sstatic.net/9UVnC.png
        let color_code = match record.level() {
            Level::Error => 31u8,
            Level::Warn => 93,
            Level::Info => 32,
            Level::Debug => 36,
            Level::Trace => 90,
        };
        println!(
            "\u{1B}[{}m\
                [{:5}] <{}:{}> {}\
                \u{1B}[0m",
            color_code,
            record.level(),
            module.unwrap(),
            line.unwrap(),
            record.args()
        );
    }

    fn flush(&self) {}
}

pub fn init() {
    Console::init_uart();

    use log::LevelFilter;
    log::set_logger(&Console).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Debug,
    });
    log::info!("Initializing console...");
}
