#![no_std]
#![no_main]
#![feature(decl_macro)]
#![feature(panic_info_message)]

mod arch;
mod console;
mod lang_items;
mod mem;

use console::println;
use polyhal::{instruction::Instruction, TrapFrame, TrapType};

#[polyhal::arch_interrupt]
fn interrupt_handler(tf: TrapFrame, trap_type: TrapType) {
    println!("{:#x?}  {:#x?}", trap_type, tf);
}

#[polyhal::arch_entry]
fn main() {
    mem::init_allocator();
    polyhal::init(None);
    println!("Hello, world!");
    Instruction::ebreak();
}
