//! Runtime context for storing heterogeneous values by type.
//!
//! This crate provides a lightweight type map with support for owned values and
//! borrowed (immutable or mutable) references. It is built on top of
//! [`better_any`](https://crates.io/crates/better_any) and uses an optimized
//! `TypeId` hasher for fast lookups.

mod context;
mod data;
mod hasher;

pub use better_any::tid;
pub use context::*;
pub use data::*;
pub use hasher::*;
