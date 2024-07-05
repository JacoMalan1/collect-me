#![warn(
    clippy::unwrap_used,
    rust_2018_idioms,
    missing_debug_implementations,
    missing_docs
)]
#![no_std]

//! Extra data-structures relating to data-lookup not defined in the standard library
//!
//! # Complexity
//!
//! The following table shows the complexity of various operations on the data-structures contained
//! within this library. For some data-structures, complexity may be different in the _worst case_
//! than in the _average case_. For such cases, the _average case_ is shown below.
//!
//! | Data-structure | Insertion    |   Removal    |    Search    |
//! | -------------- | ------------ | ------------ | ------------ |
//! |  `BinaryTree`  | _O(log(n))~_ | _O(log(n))~_ | _O(log(n))~_ |
//! |   `SkipList`   | _O(log(n))~_ | _O(log(n))~_ | _O(log(n))~_ |
//!
//! `~` - _Average_ complexity

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub use list::SkipList;
#[cfg(feature = "alloc")]
pub use tree::BinaryTree;

#[cfg(feature = "alloc")]
extern crate alloc;

/// List-like data-structures
pub mod list;
/// Tree-like data-structures
pub mod tree;
/// Modified vector data structures
pub mod vec;
