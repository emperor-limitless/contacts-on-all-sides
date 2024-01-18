use crate::{
    dm::Dm,
    inventory::Inventory,
    server::{get_server, packets},
    timer::Timer,
    weapon::Weapon,
};
use enet::Peer;
use fernet::Fernet;
use rand::Rng;
use serde_derive::*;
use std::{collections::HashMap, convert::From, fs, io::Read, path::Path, time::Instant};
fn default_weapons() -> HashMap<String, isize> {
    let mut w = HashMap::new();
    w.insert(String::from("pistol"), 1);
    w.insert(String::from("machinegun"), 1);
    w.insert(String::from("grenade_launcher"), 1);
    w
}
fn default_ammo() -> HashMap<String, isize> {
    let mut w = HashMap::new();
    w.insert(String::from("pistol"), 12);
    w.insert(String::from("machinegun"), 50);
    w.insert(String::from("grenade_launcher"), 1);
    w
}
fn default_cartridges() -> HashMap<String, isize> {
    let mut w = HashMap::new();
    w.insert(String::from("pistol"), 50);
    w.insert(String::from("machinegun"), 100);
    w.insert(String::from("grenade_launcher"), 35);
    w
}

fn default_health() -> isize {
    3000
}
fn default_can_chat() -> bool {
    true
}
fn default_hit_ping() -> bool {
    true
}
#[derive(Serialize, Deserialize)]
pub struct PlayerData {
    pub x: isize,
    pub y: isize,
    #[serde(default)]
    pub dev: bool,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub id: String,
    pub direction: usize,
    pub map: String,
    #[serde(default)]
    pub safe: bool,
    #[serde(default = "default_health")]
    pub health: isize,
    #[serde(default)]
    pub last_hit: String,
    #[serde(default, skip_serializing)]
    pub weapon: String,
    #[serde(default = "default_weapons")]
    pub weapons: HashMap<String, isize>,
    #[serde(default = "default_ammo")]
    pub ammo: HashMap<String, isize>,
    #[serde(default = "default_cartridges")]
    pub cartridges: HashMap<String, isize>,
    #[serde(default)]
    pub kills: usize,
    #[serde(default)]
    pub deaths: usize,
    #[serde(default)]
    pub potion_timer: Timer,
    #[serde(default = "default_hit_ping")]
    pub hit_ping: bool,
    #[serde(default)]
    pub agreed_to_rules: bool,
    #[serde(default)]
    pub inventory: Inventory,
    #[serde(default = "default_can_chat")]
    pub can_chat: bool,
}
impl PlayerData {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            dev: false,
            admin: false,
            id: String::new(),
            direction: 0,
            map: String::from("main"),
            safe: false,
            health: default_health(),
            can_chat: true,
            hit_ping: true,
            agreed_to_rules: false,
            last_hit: String::new(),
            weapon: String::new(),
            weapons: default_weapons(),
            kills: 0,
            deaths: 0,
            potion_timer: Timer::new(),
            inventory: Inventory::default(),
            ammo: default_ammo(),
            cartridges: default_cartridges(),
        }
    }
    pub fn get_weapon_total_ammo(&self) -> isize {
        if self.weapon == "pistol" {
            return 12;
        } else if self.weapon == "machinegun" {
            return 50;
        } else if self.weapon == "grenade_launcher" {
            return 1;
        }
        0
    }
    pub fn get_cartridges(&self) -> isize {
        if let Some(cartridges) = self.cartridges.get(&self.weapon) {
            return *cartridges;
        }
        0
    }
    pub fn get_ammo(&self) -> isize {
        if let Some(ammo) = self.ammo.get(&self.weapon) {
            return *ammo;
        }
        0
    }
    pub fn take_cartridge(&mut self, name: String, amount: isize) {
        if let Some(cartridge) = self.cartridges.get(&name) {
            if cartridge + amount <= 0 {
                self.cartridges.remove(&name);
            } else {
                self.cartridges.insert(name, cartridge + amount);
            }
        } else {
            self.cartridges.insert(name, amount);
        }
    }
    pub fn take_ammo(&mut self, name: String, amount: isize) {
        if let Some(ammo) = self.ammo.get(&name) {
            if ammo + amount <= 0 {
                self.ammo.remove(&name);
            } else {
                self.ammo.insert(name, ammo + amount);
            }
        } else {
            self.ammo.insert(name, amount);
        }
    }
    pub fn take_weapon(&mut self, name: String, amount: isize) {
        if let Some(weapon) = self.weapons.get(&name) {
            if weapon + amount <= 0 {
                self.weapons.remove(&name);
            } else {
                self.weapons.insert(name, weapon + amount);
            }
        } else {
            self.weapons.insert(name, amount);
        }
    }
}

pub struct Player {
    pub name: String,
    pub addr: Peer,
    pub packet: Option<String>,
    pub data: PlayerData,
    pub got_hit: bool,
    pub cheat_timer: Instant,
    pub cheat_time: u128,
    pub reloading: bool,
    pub reload_time: u128,
    pub reload_timer: Instant,
    pub firing: bool,
    pub fire_time: u128,
    pub fire_timer: Instant,
    pub automatic: bool,
}

impl Player {
    pub fn new(name: String, addr: Peer) -> Self {
        Self {
            name,
            addr,
            packet: None,
            data: PlayerData::new(),
            got_hit: false,
            cheat_timer: Instant::now(),
            cheat_time: 30000,
            reloading: false,
            reload_time: 0,
            reload_timer: Instant::now(),
            firing: false,
            fire_time: 0,
            fire_timer: Instant::now(),
            automatic: false,
        }
    }
    pub fn use_item(&mut self) -> anyhow::Result<()> {
        if self.data.inventory.is_empty() {
            self.say("Empty".to_string())?;
            return Ok(());
        }
        let item = self.data.inventory.item().to_string();
        if item == "health_potion" {
            let elapsed = self.data.potion_timer.elapsed();
            if elapsed < 30000 {
                self.say(format!(
                    "You can't use this, You need to wait {} seconds",
                    ((elapsed as i64 / 1000) - 30).abs()
                ))?;
                return Ok(());
            }
            if self.data.health == default_health() {
                self.say("You are already at full health".to_string())?;
                return Ok(());
            }
            self.data.potion_timer.restart();
            self.play("player/potion.mp3".to_string())?;
            self.data.health = std::cmp::min(self.data.health + 500, default_health());
        }
        self.give(&item, -1)?;
        Ok(())
    }
    pub fn cycle(&mut self, direction: usize) -> anyhow::Result<()> {
        if self.data.inventory.is_empty() {
            self.say("Empty".to_string())?;
            return Ok(());
        }
        self.data.inventory.cycle(direction);
        self.say(self.data.inventory.get_text())?;
        Ok(())
    }
    pub fn give(&mut self, item: &str, amount: isize) -> anyhow::Result<()> {
        if amount == 0 {
            return Ok(());
        }
        if item.starts_with("weapon_") {
            self.data.take_weapon(item.replace("weapon_", ""), amount);
        } else if item.starts_with("cartridge_") {
            self.data
                .take_cartridge(item.replace("cartridge_", ""), amount);
        } else if item.starts_with("ammo_") {
            self.data.take_ammo(item.replace("ammo_", ""), amount);
        } else {
            self.data.inventory.give(item, amount);
            let mut buffer = packets::Buffer::default();
            if amount >= 0 {
                buffer.text = format!("You gained {} {}", amount, item);
            } else if amount < 0 {
                buffer.text = format!("You lost {} {}", amount.abs(), item);
            }
            get_server().send(self.addr, packets::packet::Data::Buffer(buffer))?;
        }
        Ok(())
    }
    pub fn set_weapon_data(&mut self) -> anyhow::Result<()> {
        if self.data.weapon == "pistol" {
            self.reload_time = 3500;
            self.fire_time = 370;
            self.automatic = false;
        } else if self.data.weapon == "machinegun" {
            self.reload_time = 1800;
            self.fire_time = 50;
            self.automatic = true;
        } else if self.data.weapon == "grenade_launcher" {
            self.reload_time = 2800;
            self.fire_time = 800;
            self.automatic = false;
        }
        let mut weapon_data = packets::WeaponData::default();
        weapon_data.fire_time = self.fire_time.try_into()?;
        weapon_data.reload_time = self.reload_time.try_into()?;
        weapon_data.automatic = self.automatic;
        get_server().send(self.addr, packets::packet::Data::WeaponData(weapon_data))?;
        Ok(())
    }
    pub fn draw(&mut self, weapon: String) -> anyhow::Result<()> {
        if self.reloading {
            return Ok(());
        }
        self.firing = false;
        self.play(format!("weapons/{}/draw.mp3", weapon))?;
        self.say(weapon.clone())?;
        self.data.weapon = weapon;
        self.set_weapon_data()?;
        Ok(())
    }
    pub fn fire(&mut self) -> anyhow::Result<()> {
        if self.reloading
            || self.data.weapon == ""
            || self.fire_timer.elapsed().as_millis() < self.fire_time
            || self.data.safe
        {
            return Ok(());
        }
        self.fire_timer = Instant::now();
        if self.data.ammo.get(&self.data.weapon).is_none() {
            self.play(format!("weapons/{}/empty.mp3", self.data.weapon))?;
            return Ok(());
        }
        if self.automatic {
            self.firing = true;
        }
        self.play(format!(
            "weapons/{}/{}.mp3",
            self.data.weapon,
            get_server().rng.gen_range(1..=3)
        ))?;
        self.data.take_ammo(self.data.weapon.clone(), -1);
        let weapon = Weapon::new(
            self.data.x,
            self.data.y,
            self.data.direction,
            self.data.weapon.clone(),
            self.name.clone(),
            self.data.map.clone(),
        );
        get_server().weapons.push(weapon);
        Ok(())
    }
    pub fn stop_fire(&mut self) {
        if self.automatic {
            self.firing = false;
        }
    }
    pub fn reload(&mut self) -> anyhow::Result<()> {
        if self.reloading || self.data.weapon.is_empty() {
            return Ok(());
        }
        if self.data.ammo.get(&self.data.weapon).is_some() {
            return Ok(());
        }
        if self.data.cartridges.get(&self.data.weapon).is_none() {
            self.say("You don't have any cartridges left!".to_string())?;
            return Ok(());
        }
        self.reloading = true;
        self.reload_timer = Instant::now();
        self.play(format!("weapons/{}/reload.mp3", self.data.weapon))?;
        self.data.take_cartridge(self.data.weapon.clone(), -1);
        Ok(())
    }
    pub fn say(&self, text: String) -> anyhow::Result<()> {
        let mut say = packets::Say::default();
        say.text = text;
        get_server().send(self.addr, packets::packet::Data::Say(say))?;
        Ok(())
    }
    pub fn change_map(&mut self, x: isize, y: isize, map: String) -> anyhow::Result<()> {
        if map != self.data.map {
            self.data.map = map.clone();
            let parse = packets::ParseMap {
                data: std::fs::read_to_string(format!("maps/{}.map", self.data.map.clone()))?,
            };
            get_server().send(self.addr, packets::packet::Data::ParseMap(parse))?;
        }
        self.data.x = x;
        self.data.y = y;
        let mut move_client = packets::MoveClient::default();
        move_client.x = Some(x.try_into()?);
        move_client.y = Some(y.try_into()?);
        move_client.direction = Some(self.data.direction.try_into()?);
        move_client.map = map.clone();
        get_server().send(self.addr, packets::packet::Data::MoveClient(move_client))?;
        let mut move_packet = packets::Move::default();
        move_packet.x = Some(x.try_into()?);
        move_packet.y = Some(y.try_into()?);
        move_packet.direction = Some(self.data.direction.try_into()?);
        move_packet.who = self.name.clone();
        move_packet.map = map.clone();
        for (_, i) in &mut get_server().players {
            get_server().send(i.addr, packets::packet::Data::Move(move_packet.clone()))?;
        }
        Ok(())
    }
    pub fn update(&mut self) -> anyhow::Result<()> {
        if self.got_hit && self.cheat_timer.elapsed().as_millis() >= self.cheat_time {
            self.got_hit = false;
            self.cheat_timer = Instant::now();
        }
        if self.data.health <= 0 {
            self.play(format!("player/death.mp3"))?;
            self.data.health = default_health();
            self.got_hit = false;
            self.change_map(
                get_server().rng.gen_range(0..=50),
                0,
                "safe_zone".to_string(),
            )?;
            let mut buf = packets::Buffer::default();
            buf.text = format!("{} Has been killed by {}", self.name, self.data.last_hit);
            buf.name = "kills".to_string();
            get_server().broadcast(packets::packet::Data::Buffer(buf))?;
            self.data.deaths += 1;
            if let Some(player) = get_server().get_player_by_name(&self.data.last_hit) {
                player.data.kills += 1;
            }
        }
        if let Some(map) = get_server().get_map(&self.data.map) {
            if !self.data.safe && map.get_safe_zone(self.data.x, self.data.y) {
                self.data.safe = true;
                self.play("player/safe.mp3".to_string())?;
            }
            if self.data.safe && !map.get_safe_zone(self.data.x, self.data.y) {
                self.data.safe = false;
                self.play("player/unsafe.mp3".to_string())?;
            }
        }
        if self.reloading && self.reload_timer.elapsed().as_millis() >= self.reload_time {
            self.reloading = false;
            self.reload_timer = Instant::now();
            self.data
                .take_ammo(self.data.weapon.clone(), self.data.get_weapon_total_ammo());
        }
        if !self.data.safe && self.firing && self.fire_timer.elapsed().as_millis() >= self.fire_time
        {
            self.fire_timer = Instant::now();
            if self.data.ammo.get(&self.data.weapon).is_none() {
                self.play(format!("weapons/{}/empty.mp3", self.data.weapon))?;
                return Ok(());
            }
            self.play(format!(
                "weapons/{}/{}.mp3",
                self.data.weapon,
                get_server().rng.gen_range(1..=3)
            ))?;
            self.data.take_ammo(self.data.weapon.clone(), -1);
            let weapon = Weapon::new(
                self.data.x,
                self.data.y,
                self.data.direction,
                self.data.weapon.clone(),
                self.name.clone(),
                self.data.map.clone(),
            );
            get_server().weapons.push(weapon);
        }
        Ok(())
    }

    pub fn admin(&self) -> bool {
        self.data.admin || self.data.dev
    }
    pub fn play(&self, file: String) -> anyhow::Result<()> {
        let mut play = packets::Play::default();
        play.x = Some(self.data.x.try_into()?);
        play.y = Some(self.data.y.try_into()?);
        play.who = self.name.clone();
        play.map = self.data.map.clone();
        play.sound = file;
        play.self_play = Some(true);
        get_server().broadcast(packets::packet::Data::Play(play))?;
        Ok(())
    }
    pub fn self_play(&self, file: String) -> anyhow::Result<()> {
        let mut play = packets::Play::default();
        play.who = self.name.clone();
        play.map = self.data.map.clone();
        play.sound = file;
        play.self_play = Some(true);
        get_server().send(self.addr, packets::packet::Data::Play(play))?;
        Ok(())
    }

    pub fn check_account(user: String, password: String) -> anyhow::Result<bool> {
        let formatted = format!("players/{}", user);
        let player_dir = Path::new(&formatted);
        if !player_dir.exists() {
            return Ok(false);
        }
        let path = format!("players/{}/info.player", user);
        let player_path = Path::new(&path);
        if !player_path.exists() {
            return Ok(false);
        }
        let mut db = Dm::new(
            path,
            "jHYkfcq1UsvB15m7BMMlUNMNsOUlfeu-3AemrocWEJQ=".to_string(),
        );
        db.load();
        if db.exists(String::from("password")) {
            let pw = db.get("password");
            if password != pw {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub fn create(
        addr: Peer,
        user: String,
        password: String,
        mail: String,
    ) -> anyhow::Result<bool> {
        let formatted = format!("players/{}", user);
        let player_dir = Path::new(&formatted);
        if player_dir.exists() {
            return Ok(false);
        }
        fs::create_dir_all(player_dir)?;
        let path = format!("players/{}/info.player", user);
        let mut db = Dm::new(
            path,
            "jHYkfcq1UsvB15m7BMMlUNMNsOUlfeu-3AemrocWEJQ=".to_string(),
        );
        db.add(String::from("password"), password);
        db.add(String::from("mail"), mail);
        db.save()?;
        let player = Player::new(user, addr);
        player.save()?;
        Ok(true)
    }
    pub fn load(user: String, addr: Peer) -> anyhow::Result<Option<Player>> {
        let formatted = format!("players/{}", user);
        let player_dir = Path::new(&formatted);
        if !player_dir.exists() {
            return Ok(None);
        }
        let path = format!("players/{}/data.player", user);
        let player_path = Path::new(&path);
        if !player_path.exists() {
            return Ok(None);
        }
        let f = match Fernet::new("gcj57_LgWAe8KnRUqGZLf7RnfxQWs7ZzPeAkCLHZh5M=") {
            Some(f) => f,
            None => return Ok(None),
        };
        let mut file = fs::File::open(player_path)?;
        let mut text = String::new();
        file.read_to_string(&mut text)?;
        text = String::from_utf8(f.decrypt(&text)?)?;
        let mut player = Player::new(user, addr);
        player.data = serde_json::from_str(&text)?;
        Ok(Some(player))
    }
    pub fn save(&self) -> anyhow::Result<()> {
        let formatted = format!("players/{}", self.name);
        let player_dir = Path::new(&formatted);
        if !player_dir.exists() {
            println!("Atemppted saving without a file existing");
            //fs::create_dir(player_dir)?;
            return Ok(());
        }
        let path = format!("players/{}/data.player", self.name);
        let player_path = Path::new(&path);
        if !player_path.exists() {
            fs::File::create(player_path)?;
        }
        let f = match Fernet::new("gcj57_LgWAe8KnRUqGZLf7RnfxQWs7ZzPeAkCLHZh5M=") {
            Some(f) => f,
            None => return Ok(()),
        };
        let data = serde_json::to_string(&self.data)?;
        let text = f.encrypt(data.as_bytes());
        fs::write(player_path, text)?;
        Ok(())
    }
}
