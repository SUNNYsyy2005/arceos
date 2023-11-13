#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

/* pub use axlog::{pdebug, pinfo}; 
use axlog::with_color;
use axlog::ax_println;
use axlog::ColorCode;
extern crate log; */

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    //pdebug!("debug");
    //pinfo!("info");
    println!("Hello, world!");
}
