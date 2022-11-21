#![cfg(feature = "test_dungeon_kernel")]

// Needed when using the debug_msg macro
#[cfg(not(feature = "no-alloc"))]
extern crate alloc;
use host::rollup_core::{ RawRollupCore, MAX_INPUT_MESSAGE_SIZE, MAX_INPUT_SLOT_DATA_CHUNK_SIZE };
use host::runtime::Runtime;
use host::wasm_host::WasmHost;
use kernel::kernel_entry;

// Dungeon libs
use bracket_lib::terminal::main_loop;
use bracket_lib::terminal::BTermBuilder;
use bracket_lib::terminal::BError;

mod prelude {
    pub use bracket_lib::prelude::*;
    pub use crate::map_builder::*;
    pub use crate::map::*;
    pub use crate::camera::*;
    pub use crate::player::*;

    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;
}

//#[warn(unused_variables)]
mod player;
mod map_builder;
mod camera;
mod state;

use crate::prelude::{ DISPLAY_HEIGHT, DISPLAY_WIDTH };

// host max read input size: 4096
const MAX_READ_INPUT_SIZE: usize = if MAX_INPUT_MESSAGE_SIZE > MAX_INPUT_SLOT_DATA_CHUNK_SIZE {
    MAX_INPUT_MESSAGE_SIZE
} else {
    MAX_INPUT_SLOT_DATA_CHUNK_SIZE
};

/* Main function of dungeon plugin with host */

pub fn dungeon_run<Host: RawRollupCore>(host: &mut Host) {
    #[cfg(feature = "read-input")]
    match host.read_input(MAX_READ_INPUT_SIZE) {
        Some(Input::Message(message)) => {
            debug_msg!(Host, "Processing MessageData {} at level {}", message.id, message.level);
            // TODO: not use host atm
            if let Err(err) = process_dungeon(host, message.as_ref()) {
                debug_msg!(Host, "Error processing dungeon {}", err);
            }
        }
        Some(Input::Slot(_message)) => todo!("handle slot message"),
        None => (),
    }

    #[cfg(feature = "abort")]
    std::process::abort()
}

// processing main dungeon
fn process_dungeon<Host: RawRollupCore>(_host: &mut Host) -> BError {
    let context = BTermBuilder::new()
        .with_title("Dungeon Crawler")
        .with_fps_cap(30.0)
        // FIXME: these two are not working atm
        //.with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        //.with_tile_dimensions(32, 32)
        .with_resource_path("resources/")
        .with_font("dungeonfont.png", 16, 16)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .build()?;
    main_loop(context, state::State::new())
}

// plugin with kernel entry to call kernel_next()
#[cfg(feature = "test_dungeon_kernel")]
kernel_entry!(test_dungeon_run);