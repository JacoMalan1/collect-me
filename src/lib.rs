#![warn(
    clippy::unwrap_used,
    rust_2018_idioms,
    missing_debug_implementations,
    missing_docs
)]
#![no_std]

//! Extra data-structures relating to data-lookup not defined in the standard library

extern crate alloc;
/// Tree-like data-structures
pub mod tree;
