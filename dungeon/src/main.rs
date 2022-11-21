mod map;
mod player;
mod map_builder;
mod camera;
mod prelude;
mod state;

use prelude::*;

/* Main function of dungeon */

fn main() -> BError {
    // TODO Btermbuilder add host
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

    // TODO state add host
    main_loop(context, state::State::new())
}