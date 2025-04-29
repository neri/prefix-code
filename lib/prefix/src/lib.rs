#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

mod prefix;
pub use prefix::*;

pub mod bits;
pub mod stats;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EncodeError {
    InvalidInput,
    InvalidData,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DecodeError {
    InvalidInput,
    InvalidData,
    OutOfMemory,
}
