use crate::{rotation::get_2d_distance, server::get_server};
use rand::Rng;
use std::time::Instant;

#[derive(Eq, PartialEq, Debug)]
pub struct Item {
    x: isize,
    y: isize,
    name: String,
    map: String,
    timer: Instant,
}
impl Item {
    fn check(&self, x: isize, y: isize) -> bool {
        if get_2d_distance(self.x, self.y, x, y) <= 3 {
            return true;
        }
        false
    }
}

pub struct ItemSpawner {
    min_x: isize,
    max_x: isize,
    min_y: isize,
    max_y: isize,
    count: isize,
    maximum: isize,
    map: String,
    names: Vec<String>,
    items: Vec<Item>,
    timer: Instant,
    spawn_time: u128,
}
impl ItemSpawner {
    pub fn new(
        min_x: isize,
        max_x: isize,
        min_y: isize,
        max_y: isize,
        maximum: isize,
        spawn_time: u128,
        names: Vec<String>,
        map: String,
    ) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            count: 0,
            maximum,
            map,
            names,
            items: vec![],
            timer: Instant::now(),
            spawn_time,
        }
    }
    pub fn update_items(&mut self) -> anyhow::Result<()> {
        for i in 0..self.items.len() {
            if self.items[i].timer.elapsed().as_millis() >= 650 {
                self.items[i].timer = Instant::now();
                get_server().play(
                    "items/beep.mp3",
                    self.items[i].x,
                    self.items[i].y,
                    &self.items[i].map,
                )?;
            }
            for (_, p) in &mut get_server().players {
                if p.data.map == self.items[i].map && self.items[i].check(p.data.x, p.data.y) {
                    p.give(&self.items[i].name, 1)?;
                    p.play("items/gather.mp3".to_string())?;
                    self.count -= 1;
                    p.say(self.items[i].name.clone())?;
                    self.items.remove(i);
                    return Ok(());
                }
            }
        }
        Ok(())
    }
    pub fn update(&mut self) -> anyhow::Result<()> {
        if self.timer.elapsed().as_millis() >= self.spawn_time && self.count < self.maximum {
            self.timer = Instant::now();
            self.count += 1;
            let name = self.names[get_server().rng.gen_range(0..self.names.len())].clone();
            let x = get_server().rng.gen_range(self.min_x..=self.max_x);
            let y = get_server().rng.gen_range(self.min_y..=self.max_y);
            let item = Item {
                x,
                y,
                name,
                map: self.map.clone(),
                timer: Instant::now(),
            };
            self.items.push(item);
        }
        self.update_items()?;
        Ok(())
    }
}
