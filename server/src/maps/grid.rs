use super::{safe_zone::SafeZone, tile::Tile};
use crate::items::ItemSpawner;
#[derive(Default)]
pub struct Grid {
    pub max_x: usize,
    pub max_y: usize,
    tiles: Vec<Tile>,
    pub item_spawner: Vec<ItemSpawner>,
    safe_zones: Vec<SafeZone>,
    pub name: String,
}
impl Grid {
    pub fn get_tile(&self, x: isize, y: isize) -> Option<String> {
        for i in self.tiles.iter().rev() {
            if i.in_range(x, y) {
                return Some(i.tile.clone());
            }
        }
        None
    }
    pub fn get_safe_zone(&self, x: isize, y: isize) -> bool {
        for i in &self.safe_zones {
            if i.in_range(x, y) {
                return true;
            }
        }
        false
    }
    pub fn add(&mut self, min_x: isize, max_x: isize, min_y: isize, max_y: isize, tile: &str) {
        let t = Tile::new(min_x, max_x, min_y, max_y, tile);
        self.tiles.push(t);
    }
    pub fn add_safe_zone(&mut self, min_x: isize, max_x: isize, min_y: isize, max_y: isize) {
        let s = SafeZone::new(min_x, max_x, min_y, max_y);
        self.safe_zones.push(s);
    }
    pub fn update(&mut self) -> anyhow::Result<()> {
        for i in &mut self.item_spawner {
            i.update()?;
        }
        Ok(())
    }
}
