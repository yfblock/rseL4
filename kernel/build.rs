use std::env;
use std::io::Result;

#[allow(unused_macros)]
macro_rules! display {
    ($fmt:expr) => (println!("cargo:warning={}", format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("cargo:warning=", $fmt), $($arg)*));
}

fn main() {
    // write module configuration to OUT_PATH, then it will be included in the main.rs
    // let _platform = env::var("CARGO_CFG_BOARD").expect("can't find board");
    // TODO: using `_platform` isntead of `qemu` in the future
    gen_linker_script("qemu").expect("can't generate linker script");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_BOARD");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=linker.lds");
}

fn gen_linker_script(platform: &str) -> Result<()> {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("can't find target");
    let fname = format!("linker_{}_{}.lds", arch, platform);
    // Get Platform Information.
    let start_addr = match (arch.as_str(), platform) {
        ("x86_64", _) => "0xffffff8000200000",
        ("riscv64", _) => "0xffffffc080200000",
        ("aarch64", _) => "0xffffff8040800000",
        ("loongarch64", _) => "0x9000000090000000",
        _ => unimplemented!("Not found supported arch and board"),
    };

    let ld_content = std::fs::read_to_string("linker.lds")?;
    let ld_content = ld_content.replace("%START_ADDR%", start_addr);

    std::fs::write(&fname, ld_content)?;
    println!("cargo:rustc-link-arg=-Tkernel/{}", fname);
    Ok(())
}
