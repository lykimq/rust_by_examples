use crate::prelude::*;
// tile: tam gach
const NUM_TILES: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

#[allow(dead_code)]
// Represent Tiles
#[derive(Copy, Clone, PartialEq)]
/* Tiles are limited to a pre-defined set of tiles type, use enum
 because of that */
pub enum TileType {
    Wall,
    Floor,
}

/* Define a struct of Map, containing a vector of tiles*/
pub struct Map {
    pub tiles: Vec<TileType>,
}

/* Define map index: vector are indexed on a single dimension.
 This function transform map location (x,y) into vector indices */
pub fn map_idx(x: i32, y: i32) -> usize {
    (y * SCREEN_WIDTH + x) as usize
}

/* Define a constructor for the Map type */
impl Map {
    // New map
    pub fn new() -> Self {
        /* Marco vec! to create NUM_TILES to TileType::Floor;
           creating a map consisting entirely of floors */
        Self { tiles: vec!(TileType::Floor; NUM_TILES) }
    }

    /* Render the map purposes to make the map able 
   to draw itself to the screen */
    pub fn render(&self, ctx: &mut BTerm) {
        // iterating y first is faster due to memory cache usage
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let idx = map_idx(x, y);
                // determine tile type
                match self.tiles[idx] {
                    TileType::Floor => {
                        // Call set to render each map tile.
                        // Floor appear as `.` in yellow
                        ctx.set(x, y, YELLOW, BLACK, to_cp437('.'));
                    }
                    TileType::Wall => {
                        // wall as `#` in green
                        ctx.set(x, y, GREEN, BLACK, to_cp437('#'));
                    }
                }
            }
        }
    }
}