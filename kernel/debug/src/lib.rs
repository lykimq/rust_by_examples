//! Provides *debug log* which can be written to, but does not affect the host state.
//!
//! The result of writing to the debug log is *implementation specific* - it may, for
//! example, be written to a log file, or to `stdout` etc.
#![cfg_attr(target_arch = "wasm32", no_std)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]

use core::fmt::Result;
use core::marker::PhantomData;

use host::rollup_core::RawRollupCore;

/// Wrapper for writing output to the host debug log with [debug_msg!].
pub struct DebugLog<T>
where
    T: RawRollupCore,
{
    host: PhantomData<T>,
}

impl<T> DebugLog<T>
where
    T: RawRollupCore,
{
    /// Write `s` to the host debug log.  In general, you should prefer [`debug_msg!`].
    pub fn write_str(s: &str) -> Result {
        unsafe { T::write_debug(s.as_ptr(), s.len()) };
        Ok(())
    }
}

/// Write a formatted message to host debug log. Formats follow [`core::fmt`].
///
/// You can write to the debug log using `... as Host` with either:
/// - [`host::wasm_host::WasmHost`]
/// - [`mock_runtime::host::MockHost`]
/// ```
/// extern crate alloc;
/// use debug::debug_msg;
///
/// # use mock_runtime::host::{check_debug_log, MockHost as Host};
/// # use mock_runtime::state::HostState;
///
/// debug_msg!(Host, "Simple constant string");
///
/// debug_msg!(Host, "A format {} with argument {}", "test", 5);
///
/// # check_debug_log(|debug_log| {
/// #     let expected_debug = vec![
/// #         String::from("Simple constant string"),
/// #         String::from("A format test with argument 5"),
/// #     ];
///
/// #     assert_eq!(expected_debug, debug_log);
/// # })
/// ```
///
/// [`core::fmt`]: https://doc.rust-lang.org/core/fmt/index.html
#[cfg(feature = "alloc")]
#[macro_export]
macro_rules! debug_msg {
    ($host: ty, $($args: expr), *) => {
        {
            extern crate alloc;
            debug::debug_str!($host, { &alloc::format!($($args), *) });
        }
    }
}

/// Write a static message to host debug log.
///
/// You can write to the debug log using `... as Host` with either:
/// - [`host::wasm_host::WasmHost`]
/// - [`mock_runtime::host::MockHost`]
/// ```
/// extern crate alloc;
/// use debug::debug_str;
///
/// # use mock_runtime::host::{check_debug_log, MockHost as Host};
/// # use mock_runtime::state::HostState;
///
/// debug_str!(Host, "Simple constant string");
///
/// # check_debug_log(|debug_log| {
/// #     let expected_debug = vec![
/// #         String::from("Simple constant string"),
/// #     ];
/// #
/// #     assert_eq!(expected_debug, debug_log);
/// # })
/// ```
///
#[macro_export]
macro_rules! debug_str {
    ($host: ty, $msg: expr) => {{
        use core::fmt::Write;
        use debug::DebugLog;

        let result = DebugLog::<$host>::write_str($msg);

        debug_assert!(result.is_ok());
    }};
}
