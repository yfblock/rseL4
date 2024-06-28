#![feature(lazy_cell)]

use std::io::Result;
use std::env;

#[allow(unused_macros)]
macro_rules! display {
    ($fmt:expr) => (println!("cargo:warning={}", format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("cargo:warning=", $fmt), $($arg)*));
}

fn main() {
    // write module configuration to OUT_PATH, then it will be included in the main.rs
    gen_linker_script(&env::var("CARGO_CFG_BOARD").expect("can't find board"))
        .expect("can't generate linker script");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_KERNEL_BASE");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_BOARD");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=linker.lds.liquid");
}

fn gen_linker_script(platform: &str) -> Result<()> {
    let template = liquid::ParserBuilder::with_stdlib()
        .build().unwrap()
        .parse(include_str!("linker.lds.liquid")).unwrap();

    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("can't find target");
    let board = env::var("CARGO_CFG_BOARD").unwrap_or("qemu".to_string());
    let fname = format!("linker_{}_{}.lds", arch, platform);
    // Get Platform Information.
    let (output_arch, kernel_base) = match(arch.as_str(), board.as_str()) {
        ("x86_64", _) => ("i386:x86-64", "0xffffff8000200000"),
        ("riscv64", _) => ("riscv", "0xffffffc080200000"),
        ("aarch64", _) => ("aarch64", "0xffffff8040080000"),
        ("loongarch64", "2k1000") => ("loongarch64", "0x9000000098000000"),
        ("loongarch64", _) => ("loongarch64", "0x9000000090000000"),
        _ => unimplemented!("Not found supported arch and board")
    };

    let globals = liquid::object!({
        "arch": output_arch,
        "kernel_base": kernel_base,
    });  

    std::fs::write(&fname, template.render(&globals).unwrap())?;
    println!("cargo:rustc-link-arg=-Tkernel/{}", fname);
    println!("cargo:rerun-if-env-changed=CARGO_CFG_KERNEL_BASE");
    Ok(())
}
