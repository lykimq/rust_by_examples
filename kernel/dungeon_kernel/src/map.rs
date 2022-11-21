use crate::prelude::{ SCREEN_WIDTH, SCREEN_HEIGHT };
use bracket_lib::terminal::{ Point, BLACK, WHITE };

const NUM_TILES: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq)]
/* Tiles are limited to a pre-defined set of tiles type, use enum
 because of that */
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
}

/* Define map index: vector are indexed on a single dimension.
 This function transform map location (x,y) into vector indices */
pub fn map_idx(x: i32, y: i32) -> usize {
    (y * SCREEN_WIDTH + x) as usize
}

#[allow(dead_code)]
/* Define a constructor for the Map type */
impl Map {
    // New map
    pub fn new() -> Self {
        /* Marco vec! to create NUM_TILES to TileType::Floor;
           creating a map consisting entirely of floors */
        Self { tiles: vec!(TileType::Floor; NUM_TILES) }
    }

    /* Render the map purposes to make the map able to draw itself to the screen 
       add camera into the render
    */
    pub fn render(&self, ctx: &mut BTerm, camera: &Camera) {
        // this tells the library to render the first console layer, the base map
        ctx.set_active_console(0);
        // iterating y first is faster due to memory cache usage
        for y in camera.top_y..camera.bottom_y {
            for x in camera.left_x..camera.right_x {
                if self.in_bounds(Point::new(x, y)) {
                    let idx = map_idx(x, y);
                    // determine tile type
                    match self.tiles[idx] {
                        TileType::Floor => {
                            /* Call set to render each map tile. Floor appear as `.` in yellow */
                            ctx.set(
                                x - camera.left_x,
                                y - camera.top_y,
                                WHITE,
                                BLACK,
                                to_cp437('.')
                            );
                        }
                        TileType::Wall => {
                            // wall as `#` in green
                            ctx.set(
                                x - camera.left_x,
                                y - camera.top_y,
                                WHITE,
                                BLACK,
                                to_cp437('#')
                            );
                        }
                    }
                }
            }
        }
    }

    /* Add in_bounds function to be sure that player will not be 
   out of bounds */
    pub fn in_bounds(&self, point: Point) -> bool {
        point.x >= 0 && point.x < SCREEN_WIDTH && point.y >= 0 && point.y < SCREEN_HEIGHT
    }

    /* Add function to determine player can enter a tile */
    pub fn can_enter_tile(&self, point: Point) -> bool {
        // use in_bounds to be sure that players is in bounds
        // and the destination of the tile is floor
        self.in_bounds(point) && self.tiles[map_idx(point.x, point.y)] == TileType::Floor
    }

    /* try_idx determines a tile's index coordinates, and indicate an error
      condition if the requested coordinates fall outside of the map boundaries */
    pub fn try_idx(&self, point: Point) -> Option<usize> {
        if !self.in_bounds(point) { None } else { Some(map_idx(point.x, point.y)) }
    }
}