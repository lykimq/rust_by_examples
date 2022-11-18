#![cfg(feature = "test_tx_kernel")]

//needed when using the debug_msg marco
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

/* Define counter increment function inside InboxMessage */
#[derive(Debug, PartialEq, Eq)]
pub struct InboxMessage {
    counter: i64,
}

impl InboxMessage {
    pub fn increment_counter(&mut self) {
        self.counter += 1;
    }

    pub fn counter(&self) -> i64 {
        self.counter
    }
}

/* Entrypoint of the `transactions` kernel */

pub fn transaction_runs<Host: RawRollupCore>(host: &mut Host) {
    // Read input
    #[cfg(feature = "testing")]
    let output = {
        // Load input message: host.read_input
        match host.read_input(READ_BUFFER_SIZE) {
            // Input from Message
            Some(Input::Message(message)) => {
                #[cfg(feature = "write-debug")]
                debug_msg!(Host, "Processing Messagedata {:?}", message.as_ref());

                #[cfg(feature = "write-output")]
                if let Err(err) = process_header_payload(host, message.as_ref()) {
                    debug_msg!(Host, "Error processing {}", err)
                }
            }

            // Input from Slot
            Some(Input::Slot(_message)) => todo!("handle slot message"),
            None => (),
        }
    };

    #[cfg(feature = "abort")]
    std::process::abort()
}

enum CounterError {
    #[error("Counter too large: {0}")] CounterAmount(#[from] TryFromBigIntError<()>),
}

/* Define a process_header_payload take a host and a message */
fn process_header_payload<Host: RawRollupCore>(host: &mut Host) -> Result<(), CounterError> {
    let counter_incr = counter.increment_counter();
    Ok(())
}

/* Test 
    Run: cargo test
*/
#[test]
fn deposit() {
    #[cfg(feature = "test_tx_kernel")]
    /* 1. call the transaction run with the `kernel_next(transaction_runs)*/
    kernel_entry_simpl!(transaction_runs);

    /* From this I can call counter or anything */
    /* 2.1 Prepare the input for deposit */

    /* 2. Deposit message */
    let deposit = InboxMessage {
        counter, // this counter will increase
    };
    let mut deposit_message = Vec::new();
    //deposit.bin_write(&mut deposit_message).unwrap();
    deposit.write_output(&mut deposit_message).unwrap();
}