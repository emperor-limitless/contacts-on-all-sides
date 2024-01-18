use crate::{context::GameContext, game::packets};
use rand::Rng;
use std::ops::RangeInclusive;

pub struct Teleporter {
    pub min_x: isize,
    pub max_x: isize,
    pub min_y: isize,
    pub max_y: isize,
    pub end_x: isize,
    pub end_y: isize,
    pub map_name: String,
    pub range_x: Option<RangeInclusive<isize>>,
    pub range_y: Option<RangeInclusive<isize>>,
}

impl Teleporter {
    pub fn new(
        min_x: isize,
        max_x: isize,
        min_y: isize,
        max_y: isize,
        end_x: isize,
        end_y: isize,
        map_name: String,
        range_x: Option<RangeInclusive<isize>>,
        range_y: Option<RangeInclusive<isize>>,
    ) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            end_x,
            end_y,
            map_name,
            range_x,
            range_y,
        }
    }
    pub fn in_range(&self, x: isize, y: isize) -> bool {
        if x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y {
            return true;
        }
        false
    }
    pub fn teleport(&self, ctx: &mut GameContext) -> anyhow::Result<()> {
        let mut teleport = packets::Teleport::default();
        let mut x = self.end_x;
        let mut y = self.end_y;
        if let Some(rx) = &self.range_x {
            x = ctx.rng.gen_range(rx.clone());
        }
        if let Some(ry) = &self.range_y {
            y = ctx.rng.gen_range(ry.clone());
        }
        teleport.x = x.try_into()?;
        teleport.y = y.try_into()?;
        teleport.map = self.map_name.clone();
        if let Some(client) = &mut ctx.client {
            client.send(packets::packet::Data::Teleport(teleport))?;
        }
        Ok(())
    }
}
