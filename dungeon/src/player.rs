use crate::prelude::*;

pub struct Player {
    pub position: Point,
}

/* constructor of player */
impl Player {
    // Create new player at a position
    pub fn new(position: Point) -> Self {
        Self { position }
    }

    // A render function as in map, to draw player by itself
    pub fn render(&self, ctx: &mut BTerm) {
        /* calculate the screen position of the player
           and use set to draw `@` symbol at that screen location */
        ctx.set(self.position.x, self.position.y, WHITE, BLACK, to_cp437('@'))
    }

    /* move player according to the keyboard command from the user */
    pub fn update(&mut self, ctx: &mut BTerm, map: &Map) {
        if let Some(key) = ctx.key {
            // delta store the key pressed by user
            let delta = match key {
                VirtualKeyCode::Left => Point::new(-1, 0),
                VirtualKeyCode::Right => Point::new(1, 0),
                VirtualKeyCode::Up => Point::new(0, -1),
                VirtualKeyCode::Down => Point::new(0, 1),
                _ => Point::zero(),
            };

            /* calculate the player's new position */
            let new_position = self.position + delta;
            if map.can_enter_tile(new_position) {
                self.position = new_position;
            }
        }
    }
}