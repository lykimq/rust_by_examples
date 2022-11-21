mod map;
mod player;
mod map_builder;
mod camera;
mod prelude;
mod state;

use prelude::*;

extern crate alloc;
use host::rollup_core::{ RawRollupCore, MAX_INPUT_MESSAGE_SIZE, MAX_INPUT_SLOT_DATA_CHUNK_SIZE };

const MAX_READ_INPUT_SIZE: usize = if MAX_INPUT_MESSAGE_SIZE > MAX_INPUT_SLOT_DATA_CHUNK_SIZE {
    MAX_INPUT_MESSAGE_SIZE
} else {
    MAX_INPUT_SLOT_DATA_CHUNK_SIZE
};

/* Main function of dungeon */

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
}

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