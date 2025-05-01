use core::arch::global_asm;

use crate::config::{KERNEL_STACK_BITS, MAX_NUM_NODES};

pub const STACK_SIZE: usize = 0x10000;

pub static mut KERNEL_STACK_ALLOC: [[u8; bit!(KERNEL_STACK_BITS)]; MAX_NUM_NODES] = [[0; _]; _];

global_asm!("
    .section .bss
    .global  kernel_boot_stack
    .global  kernel_boot_stack_top
    kernel_boot_stack:
    .space   {stack_size}
    .size  kernel_boot_stack, .-kernel_boot_stack
    kernel_boot_stack_top:",
    stack_size = const STACK_SIZE,
);
