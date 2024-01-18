use rand::Rng;

use crate::{
    context::GameContext,
    game::{packets, packets::packet::Data},
    maps::grid::Grid,
    timer::Timer,
};

use winit::event::VirtualKeyCode;
#[derive(Eq, PartialEq, Clone)]
pub enum PlayerState {
    Jumping,
    Falling,
    Landing,
    OnGround,
}
pub struct Player {
    pub map: Grid,
    pub x: isize,
    pub y: isize,
    pub name: String,
    pub direction: usize,
    pub walk_timer: Timer,
    pub walk_time: u64,
    pub jump_limit_timer: Timer,
    pub jumped: bool,
    pub state: PlayerState,
    pub gravity_range: usize,
    pub gravity_timer: Timer,
    pub gravity_time: u64,
    pub gravity_max: usize,
    pub admin: bool,
    pub dev: bool,
    beacon_timer: Timer,
    current_zone: String,
}

impl Player {
    pub fn new(name: String, admin: bool, dev: bool) -> Self {
        Self {
            x: 0,
            y: 0,
            name,
            map: Grid::default(),
            current_zone: String::new(),
            jump_limit_timer: Timer::new(),
            jumped: false,
            direction: 0,
            beacon_timer: Timer::new(),
            walk_timer: Timer::new(),
            walk_time: 180,
            state: PlayerState::OnGround,
            gravity_range: 0,
            gravity_timer: Timer::new(),
            gravity_time: 70,
            gravity_max: 5,
            admin,
            dev,
        }
    }
    pub fn sort(&self, players: &Vec<Player>) -> Vec<Player> {
        let mut new_players = vec![];
        for i in players {
            if i.map.name == self.map.name {
                let mut player = Player::new(i.name.clone(), false, false);
                player.x = i.x;
                player.y = i.y;
                player.map.name = i.map.name.clone();
                new_players.push(player);
            }
        }
        new_players.sort_by(|a, b| {
            let dist_a = ((a.x - self.x).pow(2) + (a.y - self.y).pow(2)) as f64;
            let dist_b = ((b.x - self.x).pow(2) + (b.y - self.y).pow(2)) as f64;
            dist_a.partial_cmp(&dist_b).unwrap()
        });
        new_players
    }
    pub fn update(&mut self, ctx: &mut GameContext, is_main: bool) -> anyhow::Result<()> {
        if self.jumped && self.jump_limit_timer.elapsed() >= 2400 {
            self.jump_limit_timer.restart();
            self.jumped = false;
        }
        if is_main {
            if let Some(zone) = self.get_zone() {
                if zone != self.current_zone {
                    self.current_zone = zone;
                    ctx.speaker.speak(self.current_zone.clone(), true)?;
                }
            } else {
                if self.current_zone != String::new() {
                    self.current_zone = String::new();
                    ctx.speaker.speak("Uncharted area", true)?;
                }
            }
        }
        if ctx.config.data.get("beacon").is_none() && !is_main {
            if self.beacon_timer.elapsed() >= 600 {
                self.beacon_timer.restart();
                ctx.sound.play_3d(
                    "sounds/player/beacon.mp3",
                    self.x.try_into()?,
                    self.y.try_into()?,
                    false,
                );
            }
        }
        Ok(())
    }
    pub fn is_admin(&self) -> bool {
        self.admin || self.dev
    }
    pub fn check_movement(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if ctx.input.held_alt() {
            return Ok(());
        }
        if ctx.input.key_held(VirtualKeyCode::Left) {
            self.walk_timer.restart();
            self.step(1, ctx)?;
        } else if ctx.input.key_held(VirtualKeyCode::Right) {
            self.walk_timer.restart();
            self.step(0, ctx)?;
        }
        if let Some(tile) = self.map.get_tile(self.x, self.y + 1) {
            if tile != "" && self.state == PlayerState::OnGround {
                if ctx.input.key_held(VirtualKeyCode::Up) {
                    self.walk_timer.restart();
                    self.step(2, ctx)?;
                }
            }
        }
        if self.y > 0 {
            if let Some(tile) = self.map.get_tile(self.x, self.y - 1) {
                if tile != "" && self.state == PlayerState::OnGround {
                    if ctx.input.key_held(VirtualKeyCode::Down) {
                        self.walk_timer.restart();
                        self.step(3, ctx)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn gravity_check(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.gravity_timer.elapsed() >= self.gravity_time {
            match self.state {
                PlayerState::Falling => {
                    self.gravity_timer.restart();
                    self.y -= 1;
                    self.gravity_range += 1;
                    if self.y == 0 || self.current_tile().is_some() {
                        self.play_land(ctx)?;
                        self.gravity_range = 0;
                        self.state = PlayerState::OnGround;
                    }
                    let mut move_packet = packets::Move::default();
                    move_packet.x = Some(self.x.try_into()?);
                    move_packet.y = Some(self.y.try_into()?);
                    move_packet.silent = Some(true);
                    if let Some(client) = ctx.client.as_mut() {
                        client.send(packets::packet::Data::Move(move_packet))?;
                    }
                }
                PlayerState::Jumping => {
                    self.gravity_timer.restart();
                    self.gravity_range += 1;
                    self.y += 1;
                    if self.gravity_range >= self.gravity_max {
                        self.gravity_range = 0;
                        self.state = PlayerState::Landing;
                    }
                    if let Some(tile) = self.current_tile() {
                        if tile.contains("wall") {
                            self.gravity_range = 0;
                            self.state = PlayerState::Landing;
                            self.y -= 1;
                            ctx.sound.play(&format!("sounds/walls/{}.mp3", tile), false);
                        }
                    }
                    let mut move_packet = packets::Move::default();
                    move_packet.x = Some(self.x.try_into()?);
                    move_packet.y = Some(self.y.try_into()?);
                    move_packet.silent = Some(true);
                    if let Some(client) = ctx.client.as_mut() {
                        client.send(packets::packet::Data::Move(move_packet))?;
                    }
                }
                PlayerState::Landing => {
                    self.gravity_timer.restart();
                    self.gravity_range += 1;
                    self.y -= 1;
                    if self.gravity_range >= self.gravity_max {
                        if self.current_tile().is_some() {
                            self.state = PlayerState::OnGround;
                            self.play_land(ctx)?;
                        } else {
                            self.state = PlayerState::Falling;
                            ctx.sound.play("sounds/player/fall.mp3", false);
                            let mut play = packets::Play::default();
                            play.sound = String::from("player/fall.mp3");
                            play.x = Some(self.x.try_into()?);
                            play.y = Some(self.y.try_into()?);
                            if let Some(client) = ctx.client.as_mut() {
                                client.send(packets::packet::Data::Play(play))?;
                            }
                        }
                        self.walk_time = 180;
                        self.gravity_range = 0;
                        let mut move_packet = packets::Move::default();
                        move_packet.x = Some(self.x.try_into()?);
                        move_packet.y = Some(self.y.try_into()?);
                        move_packet.silent = Some(true);
                        if let Some(client) = ctx.client.as_mut() {
                            client.send(packets::packet::Data::Move(move_packet))?;
                        }
                        return Ok(());
                    }
                    if self.current_tile().is_some() {
                        self.gravity_range = 0;
                        self.state = PlayerState::OnGround;
                        self.walk_time = 180;
                        self.play_land(ctx)?;
                    }
                }
                PlayerState::OnGround => {
                    if self.y != 0 && self.current_tile().is_none() {
                        self.state = PlayerState::Falling;
                        ctx.sound.play("sounds/player/fall.mp3", false);
                        let mut play = packets::Play::default();
                        play.sound = String::from("player/fall.mp3");
                        play.x = Some(self.x.try_into()?);
                        play.y = Some(self.y.try_into()?);
                        if let Some(client) = ctx.client.as_mut() {
                            client.send(packets::packet::Data::Play(play))?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub fn jump(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.map.get_tile(self.x, self.y + 1).is_some()
            || self.state != PlayerState::OnGround
            || self.jumped
        {
            return Ok(());
        }
        self.jumped = true;
        self.jump_limit_timer.restart();
        self.state = PlayerState::Jumping;
        self.walk_time = 80;
        ctx.sound.play("sounds/player/jump.mp3", false);
        let mut play = packets::Play::default();
        play.sound = String::from("player/jump.mp3");
        play.x = Some(self.x.try_into()?);
        play.y = Some(self.y.try_into()?);
        if let Some(client) = ctx.client.as_mut() {
            client.send(packets::packet::Data::Play(play))?;
        }
        Ok(())
    }
    pub fn play_land(&self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if let Some(tile) = self.current_tile() {
            if tile.contains("wall") {
                ctx.sound.play(&format!("sounds/walls/{}.mp3", tile), false);
            } else if self.gravity_range < 10 {
                ctx.sound
                    .play(&format!("sounds/steps/{}/land.mp3", tile), false);
                let mut play = packets::Play::default();
                play.sound = format!("steps/{}/land.mp3", tile);
                play.x = Some(self.x.try_into()?);
                play.y = Some(self.y.try_into()?);
                if let Some(client) = ctx.client.as_mut() {
                    client.send(packets::packet::Data::Play(play))?;
                }
            } else {
                ctx.sound
                    .play(&format!("sounds/steps/{}/hardland.mp3", tile), false);
                let mut play = packets::Play::default();
                play.sound = format!("steps/{}/hardland.mp3", tile);
                play.x = Some(self.x.try_into()?);
                play.y = Some(self.y.try_into()?);
                if let Some(client) = ctx.client.as_mut() {
                    client.send(packets::packet::Data::Play(play))?;
                }
            }
        }
        Ok(())
    }
    pub fn get_zone(&self) -> Option<String> {
        self.map.get_zone(self.x, self.y)
    }
    pub fn current_tile(&self) -> Option<String> {
        self.map.get_tile(self.x, self.y)
    }
    pub fn step(&mut self, direction: usize, ctx: &mut GameContext) -> anyhow::Result<()> {
        if direction != self.direction {
            self.direction = direction;
            let mut move_packet = packets::Move::default();
            move_packet.direction = Some(direction.try_into()?);
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::Move(move_packet))?;
            }
        }
        if direction == 0 {
            if self.x < self.map.get_max_x() {
                self.x += 1;
                if let Some(tile) = self.current_tile() {
                    if tile.contains("wall") {
                        self.x -= 1;
                        ctx.sound.play(&format!("sounds/walls/{}.mp3", tile), false);
                        let mut play = packets::Play::default();
                        play.x = Some(self.x.try_into()?);
                        play.y = Some(self.y.try_into()?);
                        play.map = self.map.name.clone();
                        play.sound = format!("walls/{}.mp3", tile);
                        if let Some(client) = ctx.client.as_mut() {
                            client.send(Data::Play(play))?;
                        }
                        return Ok(());
                    }
                    ctx.sound.play(
                        &format!("sounds/steps/{}/step{}.mp3", tile, ctx.rng.gen_range(1..=5)),
                        false,
                    );
                    let mut move_packet = packets::Move::default();
                    move_packet.x = Some(self.x.try_into()?);
                    move_packet.y = Some(self.y.try_into()?);
                    if let Some(client) = ctx.client.as_mut() {
                        client.send(Data::Move(move_packet))?;
                    }
                }
            }
        }
        if direction == 1 {
            if self.x != 0 {
                self.x -= 1;
                if let Some(tile) = self.current_tile() {
                    if tile.contains("wall") {
                        self.x += 1;
                        ctx.sound.play(&format!("sounds/walls/{}.mp3", tile), false);
                        let mut play = packets::Play::default();
                        play.x = Some(self.x.try_into()?);
                        play.y = Some(self.y.try_into()?);
                        play.map = self.map.name.clone();
                        play.sound = format!("walls/{}.mp3", tile);
                        if let Some(client) = ctx.client.as_mut() {
                            client.send(Data::Play(play))?;
                        }
                        return Ok(());
                    }
                    ctx.sound.play(
                        &format!("sounds/steps/{}/step{}.mp3", tile, ctx.rng.gen_range(1..=5)),
                        false,
                    );
                    let mut move_packet = packets::Move::default();
                    move_packet.x = Some(self.x.try_into()?);
                    move_packet.y = Some(self.y.try_into()?);
                    if let Some(client) = ctx.client.as_mut() {
                        client.send(Data::Move(move_packet))?;
                    }
                }
            }
        }
        if direction == 2 {
            if self.y < 1000000000 {
                self.y += 1;
                if let Some(tile) = self.current_tile() {
                    if tile.contains("wall") {
                        self.y -= 1;
                        ctx.sound.play(&format!("sounds/walls/{}.mp3", tile), false);
                        let mut play = packets::Play::default();
                        play.x = Some(self.x.try_into()?);
                        play.y = Some(self.y.try_into()?);
                        play.map = self.map.name.clone();
                        play.sound = format!("walls/{}.mp3", tile);
                        if let Some(client) = ctx.client.as_mut() {
                            client.send(Data::Play(play))?;
                        }
                        return Ok(());
                    }
                    ctx.sound.play(
                        &format!("sounds/steps/{}/step{}.mp3", tile, ctx.rng.gen_range(1..=5)),
                        false,
                    );
                    let mut move_packet = packets::Move::default();
                    move_packet.x = Some(self.x.try_into()?);
                    move_packet.y = Some(self.y.try_into()?);
                    if let Some(client) = ctx.client.as_mut() {
                        client.send(Data::Move(move_packet))?;
                    }
                }
            }
        }
        if direction == 3 {
            if self.y != 0 {
                self.y -= 1;
                if let Some(tile) = self.current_tile() {
                    if tile.contains("wall") {
                        self.y += 1;
                        ctx.sound.play(&format!("sounds/walls/{}.mp3", tile), false);
                        let mut play = packets::Play::default();
                        play.x = Some(self.x.try_into()?);
                        play.y = Some(self.y.try_into()?);
                        play.map = String::from("main");
                        play.sound = format!("walls/{}.mp3", tile);
                        if let Some(client) = ctx.client.as_mut() {
                            client.send(Data::Play(play))?;
                        }
                        return Ok(());
                    }
                    ctx.sound.play(
                        &format!("sounds/steps/{}/step{}.mp3", tile, ctx.rng.gen_range(1..=5)),
                        false,
                    );
                    let mut move_packet = packets::Move::default();
                    move_packet.x = Some(self.x.try_into()?);
                    move_packet.y = Some(self.y.try_into()?);
                    if let Some(client) = ctx.client.as_mut() {
                        client.send(Data::Move(move_packet))?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn distance(player1: &Player, player2: &Player) -> String {
    let x = (player1.x as isize - player2.x as isize).abs();
    let y = (player1.y as isize - player2.y as isize).abs();
    let mut ablo = "";
    let mut leftright = "";
    if player2.x < player1.x {
        leftright = "On the right, ";
    } else if player2.x > player1.x {
        leftright = "On the left, ";
    }
    if player2.y < player1.y {
        ablo = "above, ";
    } else if player2.y > player1.y {
        ablo = "Below, ";
    }
    let total = x + y;
    format!(
        "{}{}{} tiles away, At {}, {}",
        ablo, leftright, total, player1.x, player1.y
    )
}
