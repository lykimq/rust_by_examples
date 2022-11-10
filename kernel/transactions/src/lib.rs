/* A copy of the transaction kernel but simplify
https://gitlab.com/tezos/kernel/-/blob/main/kernel_core/src/lib.rs 
*/

extern crate alloc;

/* Make a simple copy version of `memory.rs`.
   https://gitlab.com/tezos/kernel/-/blob/main/kernel_core/src/memory.rs
*/
pub mod memory;

use host::input::Input;
use host::rollup_core::{ RawRollupCore, MAX_INPUT_MASSAGE_SIZE, MAX_INPUT_SLOT_DATA_CHUNK_SIZE };

use crate::memory::Memory;

const MAX_READ_INPUT_SIZE: usize = if MAX_INPUT_MASSAGE_SIZE > MAX_INPUT_SLOT_DATA_CHUNK_SIZE {
    MAX_INPUT_MASSAGE_SIZE
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
    if let Some(input) = host.read_input(MAX_READ_INPUT_SIZE) {
        match input {
            Input::Message(message) => todo!("handle message"),
            Input::Slot(_message) => todo!("handle slot message"),
        }
    }
}

/* Define the `kernel_next` for the transactions kernel */
#[cfg(feature = "tx-kernel")]
pub mod tx_kernel {
    use crate::transactions_run;
    use kernel::kernel_entry;
    kernel_entry!(transactions_run);
}