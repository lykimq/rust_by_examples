#![cfg(feature = "test_dungeon_kernel")]

#[cfg(not(feature = "no-alloc"))]
extern crate alloc;

use host::input::{ Input, MessageData, SlotData };
use host::rollup_core::RawRollupCore;
use host::runtime::Runtime;
use host::wasm_host::WasmHost;
use kernel::kernel_entry;

extern crate dungeon;

pub const READ_BUFFER_SIZE: usize = 4096;

pub fn test_dungeon_run<Host: RawRollupCore>(host: &mut Host) {
    // Read input
    #[cfg(feature = "read-input")]
    let output = {
        match host.read_input(READ_BUFFER_SIZE) {
            Some(Input::Message(_data)) => todo!("handle later"),
            Some(Input::Slot(_data)) => todo!("handle later"),
            None => (),
        }
    };

    #[cfg(feature = "abort")]
    std::process::abort()
}

#[cfg(feature = "test_dungeon_kernel")]
kernel_entry!(test_dungeon_run);