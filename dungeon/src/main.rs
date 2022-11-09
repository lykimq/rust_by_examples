mod map;
mod player;

mod prelude {
    pub use bracket_lib::prelude::*;
    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    pub use crate::map::*;
    pub use crate::player::*;
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
        Self {
            map: Map::new(),
            // place player in the center of the map
            player: Player::new(Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2)),
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