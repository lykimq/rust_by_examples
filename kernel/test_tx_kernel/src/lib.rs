#![cfg(feature = "test_tx_kernel")]

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
use kernel::kernel_entry_simpl;

pub const READ_BUFFER_SIZE: usize = 4096;

/* Test Kernel
 Entrypoint for test kernel.
 - Read input. it can read input and write output to both the kernel output and log
*/

pub fn test_kernel_run<Host: RawRollupCore>(host: &mut Host) {
    #[cfg(feature = "read-input")]
    let output = {
        match host.read_input(READ_BUFFER_SIZE) {
            // Read input from Slot
            Some(Input::Slot(data)) => {
                #[cfg(feature = "write-debug")]
                debug_msg!(Host, "{:?}", data.as_ref());

                #[cfg(feature = "write-output")]
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
}

#[cfg(feature = "test_tx_kernel")]
// This is called from the kernel_entry
kernel_entry_simpl!(test_kernel_run);