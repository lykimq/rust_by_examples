//use crate::prelude::*;
use bracket_lib::terminal::Point;
use crate::map::{ map_idx, TileType, Map };
use bracket_lib::random::RandomNumberGenerator;
use bracket_lib::terminal::Rect;
use crate::prelude::{ SCREEN_HEIGHT, SCREEN_WIDTH };

// it is maximum number of rooms in a dungeon
const NUM_ROOMS: usize = 20;

/* Map builder is a structure to hanlde walls, create
random place rooms, connect those rooms with corridors and build
a dungeon for player to explore */
pub struct MapBuilder {
    /* map is it own map of this mapbuilder,it will work on its 
       copy and then pass the result to the game */
    pub map: Map,
    /* rooms vector is a list of rooms that will be added to the map.
       Each room represented with the `Rect` structure from bracklet-lib
     */
    pub rooms: Vec<Rect>,
    /* player_start stores the location at which the player enters the map */
    pub player_start: Point,
}

impl MapBuilder {
    /* fill the map with walls */
    fn fill(&mut self, tile: TileType) {
        /* This function obtains a mutable iterator
           - iter_mut(): mutable iterator
           - for_each(): change every tile into a wall
           - *t: is a de-reference
           the iterator passes `t` as a reference - an &TileType
           - De-reference: indicates that you want to write to the 
           referenced variable, not the reference itself
         */
        self.map.tiles.iter_mut().for_each(|t| {
            *t = tile;
        })
    }

    /* Carving rooms, rooms should not be overlap, and we want to
       create NUM_ROOMS locations. The location of each room is random */
    fn build_random_rooms(&mut self, rng: &mut RandomNumberGenerator) {
        while self.rooms.len() < NUM_ROOMS {
            /* generate randomly positioned room with random sizes
               - range(): produces a random number within the provided min and max ranges
            */
            let room = Rect::with_size(
                rng.range(1, SCREEN_WIDTH - 10),
                rng.range(1, SCREEN_HEIGHT - 10),
                rng.range(2, 10),
                rng.range(2, 10)
            );
            /* Test the new room against each placed room,
               flag it as overlapped if rooms intersect */
            let mut overlap = false;
            for r in self.rooms.iter() {
                if r.intersect(&room) {
                    overlap = true;
                }
            }
            if !overlap {
                /* - for_each(): provided closure on every x/y coordinate inside
                     the rectangle it represented
                    If the rooms don't overlap:
                    - check that they are within the map boundaries and 
                    - set their contains to floors
                */
                room.for_each(|p| {
                    if p.x > 0 && p.x < SCREEN_WIDTH && p.y > 0 && p.y < SCREEN_HEIGHT {
                        let idx = map_idx(p.x, p.y);
                        self.map.tiles[idx] = TileType::Floor;
                    }
                });

                self.rooms.push(room);
            }
        }
    }

    /* Carving Corridors: corridors with a horizontal and vertical section,
       joined by a single corner. Create a vertical tunnel between two points
       on the map
    */
    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        use std::cmp::{ min, max };
        /* find the lowest and highest of a pair of values, the starting position.
           It then iterates `y` from the start to the end of the corridor,
           carving the tunnel along the way
        */
        for y in min(y1, y2)..=max(y1, y2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    /* Second function to carve the corridor, the horizontal_tunnel.
       It is the same as the vertical tunnel but traversing the `x` axis instead
       of `y` axis
     */
    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        use std::cmp::{ min, max };
        for x in min(x1, x2)..=max(x1, x2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    /* Now we build the corridors */
    fn build_corridors(&mut self, rng: &mut RandomNumberGenerator) {
        let mut rooms = self.rooms.clone();
        /* - sort_by(): sort their contents, this function requires a closure -
           an inline function `cmp()` on two elements of the vector's contents.
           - Sorting the rooms by their center point before allocating corridors,
            this will make corridors connect adjacent rooms and not snake across the 
            whole maps.

            - sort_by() sends pairs of rooms to the closure `cmp`.
            - the closure `cmp` receives these as `a` and `b`:
              + a.center().x: finds the x coordinate of room A.
              this is then compared via the `cmp()` function with the center of room B.
        */
        rooms.sort_by(|a, b| a.center().x.cmp(&b.center().x));

        /* enumberate(): counts items in iterator and includes them as the first 
           entry in a tuples. 
           - skip(1): allows us to ignore the first one
         */
        for (i, room) in rooms.iter().enumerate().skip(1) {
            /* obtain the center position (as `Point` types) of the current
               and previous room. This is the reason why we skip the first room, 
               because the previous of the first would be invalid. */
            let prev = rooms[i - 1].center();
            let new = room.center();

            /* randomly dig the horizontal and vertical parts of the corridor
            and vice versa */
            if rng.range(0, 2) == 1 {
                self.apply_horizontal_tunnel(prev.x, new.x, prev.y);
                self.apply_vertical_tunnel(prev.y, new.y, new.x);
            } else {
                self.apply_vertical_tunnel(prev.y, new.y, prev.x);
                self.apply_horizontal_tunnel(prev.x, new.x, new.y);
            }
        }
    }

    /* Build the map and place the player */
    pub fn new(rng: &mut RandomNumberGenerator) -> Self {
        let mut mb = MapBuilder {
            map: Map::new(),
            rooms: Vec::new(),
            player_start: Point::zero(),
        };
        // fill the wall
        mb.fill(TileType::Wall);
        // build random rooms
        mb.build_random_rooms(rng);
        // build random corridors
        mb.build_corridors(rng);
        /* set the player start at the first room in the rooms list,
           to ensure that they start in a valid and walkable tile
        */
        mb.player_start = mb.rooms[0].center();
        mb
    }
}