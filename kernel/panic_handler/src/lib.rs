//! Definition of panic handler used by *kernel* when targetting wasm.
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![forbid(unsafe_code)]

extern crate alloc;

use debug::debug_msg;
use std::panic::PanicInfo;
use std::string::String;

#[cfg(target_arch = "wasm32")]
use host::wasm_host::WasmHost as Host;

#[cfg(not(target_arch = "wasm32"))]
use mock_runtime::host::MockHost as Host;

/// Prints the panic info to the host's *debug log*, and then aborts.
///
/// When targeting WASM, this will be the *global* panic handler.
pub fn panic_handler(info: &PanicInfo) {
    #[cfg(feature = "debug-panic")]
    if let Some(message) = info.payload().downcast_ref::<String>() {
        debug_msg!(Host, "Kernel panic {:?} at {:?}", message, info.location());
    } else {
        let message = info.payload().downcast_ref::<&str>();
        debug_msg!(Host, "Kernel panic {:?} at {:?}", message, info.location());
    }

    // If we're testing, we want to be able to see the panic trace
    #[cfg(all(feature = "abort-on-panic", not(feature = "testing")))]
    {
        std::process::abort()
    }
}