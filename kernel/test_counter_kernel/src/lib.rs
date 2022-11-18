#![cfg(feature = "test_counter_kernel")]

// Needed when using the debug_msg macro
#[cfg(not(feature = "no-alloc"))]
extern crate alloc;

use debug::debug_msg;
use host::input::{ Input, MessageData, SlotData };
use host::rollup_core::RawRollupCore;
use host::runtime::Runtime;
use host::wasm_host::WasmHost;
use kernel::kernel_entry_simpl;

pub const READ_BUFFER_SIZE: usize = 4096;

pub struct TestCounter {
    //#[cfg(feature = "no-alloc")]
    counter: u32,
}

impl Default for TestCounter {
    fn default() -> Self {
        //#[cfg(feature = "no-alloc")]
        return Self { counter: 0 };
    }
}

/* Entrypoint of the `counter` kernel */

pub fn test_counter_run<Host: RawRollupCore>(host: &mut Host, counter: &mut TestCounter) {
    // Read input

    #[cfg(feature = "read-input")]
    let output = {
        // Load input message: host.read_input
        match host.read_input(READ_BUFFER_SIZE) {
            // Input from Message
            Some(Input::Message(data)) => {
                #[cfg(feature = "write-debug")]
                debug_msg!(Host, "{:?}", data.as_ref());

                #[cfg(feature = "write-output")]
                host.write_output(data.as_ref());
            }

            // Input from Slot
            Some(Input::Slot(data)) => {
                #[cfg(feature = "write-debug")]
                debug_msg!(Host, "{:?}", data.as_ref());

                #[cfg(feature = "write-output")]
                host.write_output(data.as_ref());
            }
            None => (),
        }
    };

    //#[cfg(feature = "no-alloc")]
    {
        counter.counter += 1;
    }

    #[cfg(feature = "abort")]
    std::process::abort()
}

#[cfg(feature = "test_counter_kernel")]
kernel_entry_simpl!(test_counter_run, TestCounter);