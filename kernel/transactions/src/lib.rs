/* A copy of the transaction kernel but simplify
https://gitlab.com/tezos/kernel/-/blob/main/kernel_core/src/lib.rs 
*/
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![forbid(unsafe_code)]

extern crate alloc;

/* Make a simple copy version of `memory.rs`.
   https://gitlab.com/tezos/kernel/-/blob/main/kernel_core/src/memory.rs
*/
pub mod memory;

use host::input::Input;
use host::rollup_core::{ RawRollupCore, MAX_INPUT_MESSAGE_SIZE, MAX_INPUT_SLOT_DATA_CHUNK_SIZE };

use debug::debug_msg;

use thiserror::Error;

use crate::memory::Memory;

const MAX_READ_INPUT_SIZE: usize = if MAX_INPUT_MESSAGE_SIZE > MAX_INPUT_SLOT_DATA_CHUNK_SIZE {
    MAX_INPUT_MESSAGE_SIZE
} else {
    MAX_INPUT_SLOT_DATA_CHUNK_SIZE
};

/* Entrypoint of the *transactions* kernel */
pub fn transactions_run<Host: RawRollupCore>(host: &mut Host) {
    // each kernel has one memory
    let mut memory = Memory::load_memory(host);
    /* if there is some input, use host.read_input to match 
       what kinds of input it is: message or a slot
     */
    match host.read_input(MAX_READ_INPUT_SIZE) {
        Some(Input::Message(message)) => {
            debug_msg!(Host, "Processing MessageData {} at level {}", message.id, message.level);

            if let Err(err) = process_header_payload(host, &mut memory, message.as_ref()) {
                debug_msg!(Host, "Error processing header payload {}", err);
            }
        }
        Some(Input::Slot(_message)) => todo!("handle slot message"),
        None => {}
    }
}

/* Define the `kernel_next` for the transactions kernel */
#[cfg(feature = "tx-kernel")]
pub mod tx_kernel {
    use crate::transactions_run;
    use kernel::kernel_entry;
    kernel_entry!(transactions_run);
}