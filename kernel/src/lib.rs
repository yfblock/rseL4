#![no_std]
#![feature(generic_arg_infer)]

#[macro_use]
extern crate hal;

#[macro_use]
pub mod console;
#[macro_use]
pub mod arch;

pub mod boot;
pub mod config;
pub mod driver;
mod lang_items;
pub mod object;
pub mod platform;
