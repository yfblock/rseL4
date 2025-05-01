#![no_std]
#![no_main]
#![feature(generic_arg_infer)]

#[macro_use]
extern crate hal;

#[macro_use]
mod console;
#[macro_use]
mod arch;

mod config;
mod driver;
mod lang_items;
mod object;
mod platform;
