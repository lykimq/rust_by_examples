mod map;
mod player;
mod map_builder;

mod prelude {
    pub use bracket_lib::prelude::*;
    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    pub use crate::map::*;
    pub use crate::player::*;
    pub use crate::map_builder::*;
}

use prelude::*;

/* Add a Map to the State object and initialiwe it State's constructor */
struct State {
    // draw a map
    map: Map,
    // player
    player: Player,
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
        }
    }
}

impl GameState for State {
    // tick() function call map render()
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        // update player position in the map
        self.player.update(ctx, &self.map);
        // render map first then player
        self.map.render(ctx);
        self.player.render(ctx);
    }
}

/* Main function of dungeon */
fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Dungeon Crawler")
        /* fps_cap: automatically tracks game speed,
           and tell OS that it can rest in between frames. */
        .with_fps_cap(30.0)
        .build()?;

    main_loop(context, State::new())
}