#![cfg_attr(not(test), no_std)]
mod macros;

pub type PFN = Option<call_conv!(fn (u8, *mut u8))>;