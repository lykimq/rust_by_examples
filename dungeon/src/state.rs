use crate::prelude::*;

pub struct State {
    map: Map,
    player: Player,
    camera: Camera,
}

impl State {
    pub fn new() -> Self {
        let mut rng = RandomNumberGenerator::new();
        let map_builder = MapBuilder::new(&mut rng);

        Self {
            map: map_builder.map,
            player: Player::new(map_builder.player_start),
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