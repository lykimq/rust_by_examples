//! Definition of the SCORU wasm host functions.
//!
//! The host exposes 'safe capabilities' as a set of **C-style APIs**.  The `host`
//! crate defines these as `extern` functions (see [rollup_core]) and is
//! responsible for providing safe wrappers which can be called from **safe rust**.
#![cfg_attr(not(feature = "testing"), no_std)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod input;
pub mod path;
pub mod rollup_core;
pub mod runtime;
pub mod wasm_host;
