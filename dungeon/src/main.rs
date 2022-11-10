mod map;
mod player;
mod map_builder;
mod camera;

mod prelude {
    pub use bracket_lib::prelude::*;
    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    /* Graphics layers */
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;
    pub use crate::map::*;
    pub use crate::player::*;
    pub use crate::map_builder::*;
    pub use crate::camera::*;
}

use prelude::*;

/* Add a Map to the State object and initialiwe it State's constructor */
struct State {
    // draw a map
    map: Map,
    // player
    player: Player,
    // add camera to state
    camera: Camera,
}

impl State {
    fn new() -> Self {
        // use MapBuilder to generate a new dungeon
        let mut rng = RandomNumberGenerator::new();
        let map_builder = MapBuilder::new(&mut rng);

        // use map and player to create map and player
        Self {
            /* version 1, map and player is empty with `.` and `@`
              map: Map::new(),
            // place player in the center of the map
            player: Player::new(Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2))*/
            /* version 2: create map and build use map_builder */
            map: map_builder.map,
            player: Player::new(map_builder.player_start),
            // add camera
            camera: Camera::new(map_builder.player_start),
        }
    }
}

impl GameState for State {
    // tick() function call map render()
    fn tick(&mut self, ctx: &mut BTerm) {
        // set the layer is base layer
        ctx.set_active_console(0);
        ctx.cls();
        // set the layer is player
        ctx.set_active_console(1);
        ctx.cls();
        // update player position in the map
        self.player.update(ctx, &self.map, &mut self.camera);
        // render map first then player
        self.map.render(ctx, &self.camera);
        self.player.render(ctx, &self.camera);
    }
}

/* Main function of dungeon */
fn main() -> BError {
    /* old graphics*/
    /*let context = BTermBuilder::simple80x50()
        .with_title("Dungeon Crawler")
        /* fps_cap: automatically tracks game speed,
           and tell OS that it can rest in between frames. */
        .with_fps_cap(30.0)
        .build()?;
    */

    /* work without graphics
    let context = BTermBuilder::simple80x50()
        .with_title("Dungeon Crawler")
        /* fps_cap: automatically tracks game speed,
           and tell OS that it can rest in between frames. */
        .with_fps_cap(30.0)
        //.with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        //.with_tile_dimensions(32, 32)
        .with_resource_path("resources/")
        .with_font("dungeonfont.png", 16, 16)
        //.with_font("terminal8x8.png", 8, 8)
        //.with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .build()?;
        */

    /* Use a new graphics layers 
      - new(): to create a generic terminal and specify attributes directly
      - with_dimensions: specifices the size of subsequent consoles you add
      - tile dimensions are the size of each character in your font file, in this case
      it is 32x32
      - the directory in which you placed the graphics files
      - the name of the font file to load, and the character dimensions. 
      - add a console using the dimensions already specified and the named tile graphics file
      - add a second console with no background so transparency shows through it

      The code creates a terminal with 2 console layers: 
      - one for the map and
      - one for the player.
      We do not rendering the whole map at once and to limit the viewpoint, this is
      why we need a camera.
    */

    /* run with cargo run --release */
    let context = BTermBuilder::new()
        .with_title("Dungeon Crawler")
        .with_fps_cap(30.0)
        // FIXME: these two are not working atm
        //.with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        //.with_tile_dimensions(32, 32)
        .with_resource_path("resources/")
        // Todo where is this file??
        .with_font("dungeonfont.png", 16, 16)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .build()?;

    main_loop(context, State::new())
}