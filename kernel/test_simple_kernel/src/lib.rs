#![cfg(feature = "test_simple_kernel")]

// needed when using the debug_msg macro
#[cfg(not(feature = "no-alloc"))]
extern crate alloc;

#[cfg(not(feature = "no-alloc"))]
use alloc::boxed::Box;
use debug::debug_msg;
use host::input::{ Input, MessageData, SlotData };
use host::rollup_core::RawRollupCore;
use host::runtime::Runtime;
use host::wasm_host::WasmHost;
use kernel::kernel_entry;

pub const READ_BUFFER_SIZE: usize = 4096;

/* Test Kernel
 Entrypoint for test kernel.
 - Read input. it can read input and write output to both the kernel output and log
*/

pub fn test_kernel_run<Host: RawRollupCore>(host: &mut Host) {
    #[cfg(feature = "read-input")]
    let output = {
        match
            // Loads the oldest input still present in the inbox of
            // the smart rollup in the transient memory of the WASM kernel.
            host.read_input(READ_BUFFER_SIZE)
        {
            // Read input from Slot
            Some(Input::Slot(data)) => {
                #[cfg(feature = "write-debug")]
                debug_msg!(Host, "{:?}", data.as_ref());

                #[cfg(feature = "write-output")]
                // Writes an in-memory buffer to the outbox of the smart rollup.
                host.write_output(data.as_ref())
            }
            // Read input from Message
            Some(Input::Message(data)) => {
                #[cfg(feature = "write-debug")]
                debug_msg!(Host, "{:?}", data.as_ref());

                #[cfg(feature = "write-output")]
                host.write_output(data.as_ref())
            }
            None => (),
        }
    };

    #[cfg(feature = "abort")]
    std::process::abort()
}

#[cfg(feature = "test_simple_kernel")]
// This is called from the kernel_entry
kernel_entry!(test_kernel_run);