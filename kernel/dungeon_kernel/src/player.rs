use bracket_lib::terminal::{ Point, BTerm, to_cp437, BLACK, WHITE };
use crate::map::Map;
use crate::camera::Camera;

pub struct Player {
    pub position: Point,
}

impl Player {
    // Create new player at a position
    pub fn new(position: Point) -> Self {
        Self { position }
    }

    /* A render function as in map, to draw player by itself.
       Add camera on render for player
    */
    pub fn render(&self, ctx: &mut BTerm, camera: &Camera) {
        /* This is specifies that you want the second layer for the player 
            0: for the base case, map
            1: for the player
        */
        ctx.set_active_console(1);
        /* calculate the screen position of the player
           and use set to draw `@` symbol at that screen location */
        ctx.set(
            self.position.x - camera.left_x,
            self.position.y - camera.top_y,
            WHITE,
            BLACK,
            to_cp437('@')
        )
    }

    /* move player according to the keyboard command from the user,
      add camera into player update
     */
    pub fn update(&mut self, ctx: &mut BTerm, map: &Map, camera: &mut Camera) {
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
                // add camera for player when he moves
                camera.on_player_move(new_position);
            }
        }
    }
}