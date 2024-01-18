#[derive(Default, Clone)]
pub struct Tile {
    min_x: isize,
    max_x: isize,
    min_y: isize,
    max_y: isize,
    pub tile: String,
}

impl Tile {
    pub fn new(min_x: isize, max_x: isize, min_y: isize, max_y: isize, tile: &str) -> Tile {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            tile: tile.to_string(),
        }
    }
    pub fn in_range(&self, x: isize, y: isize) -> bool {
        if x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y {
            return true;
        }
        false
    }
}
