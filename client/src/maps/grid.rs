use super::{ambience::Ambience, teleporter::Teleporter, tile::Tile};
use crate::context::GameContext;
use std::ops::RangeInclusive;
#[derive(Default)]
pub struct Grid {
    pub max_x: isize,
    pub max_y: isize,
    tiles: Vec<Tile>,
    zones: Vec<Zone>,
    ambiences: Vec<Ambience>,
    teleporters: Vec<Teleporter>,
    pub name: String,
}
impl Grid {
    pub fn new(max_x: isize, max_y: isize, name: String) -> Self {
        Self {
            max_x,
            max_y,
            tiles: vec![],
            zones: vec![],
            ambiences: vec![],
            teleporters: vec![],
            name,
        }
    }
    pub fn get_zone(&self, x: isize, y: isize) -> Option<String> {
        for i in self.zones.iter().rev() {
            if i.in_range(x, y) {
                return Some(i.text.clone());
            }
        }
        None
    }
    pub fn get_tile(&self, x: isize, y: isize) -> Option<String> {
        for i in self.tiles.iter().rev() {
            if i.in_range(x, y) {
                return Some(i.tile.clone());
            }
        }
        None
    }
    pub fn add(&mut self, min_x: isize, max_x: isize, min_y: isize, max_y: isize, tile: &str) {
        let t = Tile::new(min_x, max_x, min_y, max_y, tile);
        self.tiles.push(t);
    }
    pub fn add_ambience(&mut self, ambience: Ambience) {
        self.ambiences.push(ambience);
    }
    pub fn add_source(
        &mut self,
        min_x: isize,
        max_x: isize,
        min_y: isize,
        max_y: isize,
        sound_path: String,
        sound_volume: f64,
        ctx: &mut GameContext,
    ) -> anyhow::Result<()> {
        let ambience = Ambience::new(min_x, max_x, min_y, max_y, sound_path, sound_volume, ctx)?;
        self.ambiences.push(ambience);
        Ok(())
    }
    pub fn check_teleport(
        &mut self,
        x: isize,
        y: isize,
        ctx: &mut GameContext,
    ) -> anyhow::Result<()> {
        for i in &mut self.teleporters {
            if i.in_range(x, y) {
                i.teleport(ctx)?;
            }
        }
        Ok(())
    }
    pub fn grid_loop(&mut self, x: isize, y: isize, ctx: &mut GameContext) -> anyhow::Result<()> {
        for i in &mut self.ambiences {
            i.sound_loop(x, y, ctx)?;
        }
        Ok(())
    }
    pub fn add_zone(
        &mut self,
        min_x: isize,
        max_x: isize,
        min_y: isize,
        max_y: isize,
        text: String,
    ) {
        self.zones.push(Zone::new(min_x, max_x, min_y, max_y, text));
    }
    pub fn add_teleporter(
        &mut self,
        min_x: isize,
        max_x: isize,
        min_y: isize,
        max_y: isize,
        end_x: isize,
        end_y: isize,
        map_name: String,
        range_x: Option<RangeInclusive<isize>>,
        range_y: Option<RangeInclusive<isize>>,
    ) {
        self.teleporters.push(Teleporter::new(
            min_x, max_x, min_y, max_y, end_x, end_y, map_name, range_x, range_y,
        ));
    }

    pub fn get_max_x(&self) -> isize {
        self.max_x
    }
}

#[derive(Default, Clone)]
pub struct Zone {
    min_x: isize,
    max_x: isize,
    min_y: isize,
    max_y: isize,
    pub text: String,
}
impl Zone {
    pub fn new(min_x: isize, max_x: isize, min_y: isize, max_y: isize, text: String) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            text,
        }
    }
    pub fn in_range(&self, x: isize, y: isize) -> bool {
        if x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y {
            return true;
        }
        false
    }
}
