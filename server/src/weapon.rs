use crate::server::get_server;
use rand::Rng;
use std::time::Instant;

pub struct Weapon {
    damage: isize,
    speed: u128,
    range: usize,
    pos: usize,
    move_timer: Instant,
    owner: String,
    name: String,
    map: String,
    x: isize,
    y: isize,
    pub valid: bool,
    facing: usize,
}

impl Weapon {
    pub fn new(
        x: isize,
        y: isize,
        facing: usize,
        name: String,
        owner: String,
        map: String,
    ) -> Self {
        let mut damage = 0;
        let mut speed = 20;
        let mut range = 0;
        if name == "pistol" {
            damage = 210;
            range = 20;
            speed = 25;
        } else if name == "machinegun" {
            range = 30;
            speed = 10;
            damage = 30;
        } else if name == "grenade_launcher" {
            range = 60;
            speed = 5;
            damage = 400;
        }
        Self {
            damage,
            speed,
            range,
            pos: 0,
            move_timer: Instant::now(),
            owner,
            map,
            name,
            x,
            y,
            facing,
            valid: true,
        }
    }
    pub fn move_loop(&mut self) -> anyhow::Result<()> {
        if !self.valid {
            return Ok(());
        }
        if self.move_timer.elapsed().as_millis() >= self.speed {
            self.move_timer = Instant::now();
            match self.facing {
                0 => {
                    self.x += 1;
                }
                1 => {
                    self.x -= 1;
                }
                2 => {
                    self.y += 1;
                }
                3 => {
                    self.y -= 1;
                }
                _ => (),
            }
            if let Some(map) = get_server().get_map(&self.map) {
                if let Some(tile) = map.get_tile(self.x, self.y) {
                    if tile.contains("wall") {
                        self.valid = false;
                        get_server().play(
                            &format!("walls/{}.mp3", tile),
                            self.x,
                            self.y,
                            &self.map,
                        )?;
                    }
                }
            }
            self.pos += 1;
            if self.pos > self.range {
                self.valid = false;
                return Ok(());
            }
        }
        for (_, i) in &mut get_server().players {
            if !i.data.safe
                && i.name != self.owner
                && i.data.map == self.map
                && i.data.x == self.x
                && i.data.y == self.y
            {
                i.data.health -= self.damage;
                i.data.last_hit = self.owner.clone();
                i.got_hit = true;
                i.cheat_timer = Instant::now();
                i.play(format!(
                    "player/pain{}.mp3",
                    get_server().rng.gen_range(1..=3)
                ))?;
                i.play(format!(
                    "weapons/{}/hit{}.mp3",
                    self.name,
                    get_server().rng.gen_range(1..=3)
                ))?;
                if let Some(player) = get_server().get_player_by_name(&self.owner) {
                    if player.data.hit_ping {
                        player.self_play("notifications/dialog.mp3".to_string())?;
                    }
                }
                self.valid = false;
                return Ok(());
            }
        }
        Ok(())
    }
}
