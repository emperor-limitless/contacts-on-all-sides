use crate::{
    buffer::Buffer,
    context::GameContext,
    dialog::Dialog,
    input::Input,
    maps,
    maps::grid::Grid,
    menu::{Menu, MenuBuilder, MenuItem},
    network_state::NetworkState,
    player::{distance, Player, PlayerState},
    state_manager::{State, Transition},
};
use clipboard::{ClipboardContext, ClipboardProvider};
use hardware_id::get_id;
use log::warn;
use prost::Message;
use std::time::Instant;
use synthizer as syz;
use winit::event::VirtualKeyCode;
pub mod packets {
    include!(concat!(env!("OUT_DIR"), "/network.packets.rs"));
}
use packets::packet::Data;

pub struct GameExit {
    timer: Instant,
}
impl GameExit {
    pub fn new() -> Self {
        Self {
            timer: Instant::now(),
        }
    }
}
impl State<GameContext> for GameExit {
    fn on_update(
        &mut self,
        _: &mut GameContext,
        depth: usize,
    ) -> anyhow::Result<Transition<GameContext>> {
        if depth != 0 {
            return Ok(Transition::None);
        }
        if self.timer.elapsed().as_millis() >= 2500 {
            return Ok(Transition::Pop(2));
        }
        Ok(Transition::None)
    }
}
#[derive(PartialEq, Eq, Hash)]
pub enum GameState {
    Creating,
    LoggingIn,
    SettingAccount,
    None,
}
pub struct Game {
    ping_timer: Option<Instant>,
    pinging: bool,
    buffer: Buffer,
    automatic: bool,
    firing: bool,
    pub state: GameState,
    pub player: Player,
    pub players: Vec<Player>,
}

impl Game {
    pub fn new() -> anyhow::Result<Self> {
        let mut buffer = Buffer::new();
        buffer.add("chat".to_string());
        Ok(Self {
            ping_timer: None,
            pinging: false,
            buffer,
            automatic: false,
            firing: false,
            state: GameState::None,
            player: Player::new(String::new(), false, false),
            players: vec![],
        })
    }
    pub fn login(&mut self, ctx: &mut GameContext) -> anyhow::Result<Transition<GameContext>> {
        if !ctx.config.data.contains_key(&String::from("user"))
            || !ctx.config.data.contains_key(&String::from("password"))
        {
            return self.set_account(ctx);
        }
        if let Some(client) = ctx.client.as_mut() {
            client.state = NetworkState::AwaitingResponse;
            let mut login = packets::Login::default();
            login.user = ctx.config.data["user"].clone();
            login.password = ctx.config.data["password"].clone();
            login.version = env!("CARGO_PKG_VERSION").to_string();
            let id = get_id()?;
            login.id = id;
            if std::env::var("CARGO_MANIFEST_DIR").is_ok() {
                login.dev = Some(true);
            }
            client.send(Data::Login(login))?;
            ctx.speaker.speak("Logging in...", true)?;
        }
        Ok(Transition::None)
    }
    pub fn set_account(&mut self, _: &mut GameContext) -> anyhow::Result<Transition<GameContext>> {
        let i = Input::new()
            .set_title("Enter you're username".to_string())
            .set_callback(|ctx, user| {
                if user.is_empty() || user.contains(char::is_whitespace) {
                    return Ok(Transition::Pop(2));
                }
                ctx.config.data.insert("user".to_string(), user);
                let pw = Input::new()
                    .set_title("Enter you're password".to_string())
                    .set_callback(|ctx, password| {
                        if password.len() < 8 || password.is_empty() {
                            return Ok(Transition::Pop(2));
                        }
                        ctx.config
                            .data
                            .insert("password".to_string(), password.clone());
                        if let Some(client) = ctx.client.as_mut() {
                            client.state = NetworkState::AwaitingResponse;
                            ctx.speaker.speak("Logging in...", true)?;
                            let mut login = packets::Login::default();
                            login.user = ctx.config.data["user"].clone();
                            login.password = password.clone();
                            if std::env::var("CARGO_MANIFEST_DIR").is_ok() {
                                login.dev = Some(true);
                            }
                            client.send(Data::Login(login))?;
                        }
                        Ok(Transition::Pop(1))
                    });
                Ok(Transition::Replace(1, Box::new(pw)))
            });
        Ok(Transition::Push(Box::new(i)))
    }
    pub fn create(&mut self, _: &mut GameContext) -> anyhow::Result<Transition<GameContext>> {
        let user = Input::new()
            .set_title("Enter your username".to_string())
            .set_callback(|ctx, username| {
                if username == "" || username.contains(" ") {
                    let dlg = Dialog::new(
                        "Error: Username can't be empty or contain whitepsaces!".to_string(),
                    );
                    return Ok(Transition::Replace(2, Box::new(dlg)));
                }
                ctx.config.data.insert("user".to_string(), username);
                let pw = Input::new()
                    .set_title("Enter your password, At least 8 characters".to_string())
                    .set_callback(|ctx, password| {
                        if password == "" || password.contains(" ") {
                            let dlg = Dialog::new(
                                "Error: Password can't be empty or contain whitespaces!"
                                    .to_string(),
                            );
                            return Ok(Transition::Replace(2, Box::new(dlg)));
                        }
                        if password.len() < 8 {
                            let dlg = Dialog::new("Error: Password too short!".to_string());
                            return Ok(Transition::Replace(2, Box::new(dlg)));
                        }
                        ctx.config.data.insert("password".to_string(), password);
                        let mail = Input::new()
                            .set_title("Enter your email address".to_string())
                            .set_callback(|ctx, mail| {
                                if !mail.contains(".") || !mail.contains("@") {
                                    let dlg =
                                        Dialog::new("Error: Invalid email address".to_string());
                                    return Ok(Transition::Replace(2, Box::new(dlg)));
                                }
                                if let Some(client) = ctx.client.as_mut() {
                                    ctx.speaker.speak("Creating...", true)?;
                                    client.state = NetworkState::AwaitingResponse;
                                    let mut create = packets::Create::default();
                                    create.user = ctx.config.data["user"].clone();
                                    create.password = ctx.config.data["password"].clone();
                                    create.email = mail;
                                    let id = get_id()?;
                                    create.id = id;
                                    client.send(Data::Create(create))?;
                                }
                                Ok(Transition::Pop(1))
                            });
                        Ok(Transition::Replace(1, Box::new(mail)))
                    });
                Ok(Transition::Replace(1, Box::new(pw)))
            });
        Ok(Transition::Push(Box::new(user)))
    }
    pub fn process_packet(
        &mut self,
        ctx: &mut GameContext,
    ) -> anyhow::Result<Transition<GameContext>> {
        if let Some(client) = ctx.client.as_mut() {
            if let Some(packet) = client.packet.clone() {
                let buf = packet.as_slice();
                let packet = packets::Packet::decode(buf);
                if packet.is_err() {
                    return Ok(Transition::None);
                }
                match packet?.data {
                    Some(Data::Connected(connected)) => {
                        if let Some(client) = ctx.client.as_mut() {
                            client.state = NetworkState::Connected;
                            for i in connected.players {
                                let mut player = Player::new(i.name.clone(), false, false);
                                player.x = i.x.try_into()?;
                                player.y = i.y.try_into()?;
                                player.direction = i.direction.try_into()?;
                                player.map = Grid::default();
                                player.map.name = i.map;
                                self.players.push(player);
                            }
                            let mut admin = false;
                            let mut dev = false;
                            if let Some(a) = connected.admin {
                                self.buffer.add_item(
                                    "You are an admin".to_string(),
                                    "misc".to_string(),
                                    true,
                                    ctx,
                                )?;
                                admin = a;
                            }
                            if let Some(d) = connected.dev {
                                dev = d;
                            }
                            self.spawn_player(ctx.config.data["user"].clone(), admin, dev);
                        }
                    }
                    Some(Data::Close(_)) => {
                        ctx.speaker
                            .speak("Your connection has been closed!", true)?;
                        if let Some(client) = ctx.client.as_mut() {
                            client.state = NetworkState::Unconnected;
                        }
                        return Ok(Transition::PopExcept(1));
                    }
                    Some(Data::Created(_)) => {
                        if self.state == GameState::Creating {
                            let dlg = Dialog::new(
                                "Account successfully created, Press enter to continue..."
                                    .to_string(),
                            );
                            return Ok(Transition::Replace(1, Box::new(dlg)));
                        }
                    }
                    Some(Data::Error(error)) => {
                        let dlg = Dialog::new(error.reason);
                        return Ok(Transition::Replace(1, Box::new(dlg)));
                    }
                    Some(Data::MoveClient(mcl)) => {
                        if let Some(x) = mcl.x {
                            self.player.x = x.try_into()?;
                        }
                        if let Some(y) = mcl.y {
                            self.player.y = y.try_into()?;
                        }
                        if let Some(dir) = mcl.direction {
                            self.player.direction = dir.try_into()?;
                        }
                        self.player.map.name = mcl.map.clone();
                        self.player.state = PlayerState::OnGround;
                        self.player.walk_time = 180;
                        self.player.gravity_range = 0;
                    }
                    Some(Data::Move(mpc)) => {
                        /*if mpc.who == self.player.name {
                            if let Some(x) = mpc.x {
                                self.player.x = x.try_into()?;
                            }
                            if let Some(y) = mpc.y {
                                self.player.y = y.try_into()?;
                            }
                            if let Some(direction) = mpc.direction {
                                self.player.direction = direction.try_into()?;
                            }
                            self.player.map.name = mpc.map.clone();
                        }*/
                        for i in &mut self.players {
                            if mpc.who == i.name {
                                if let Some(x) = mpc.x {
                                    i.x = x.try_into()?;
                                }
                                if let Some(y) = mpc.y {
                                    i.y = y.try_into()?;
                                }
                                if let Some(direction) = mpc.direction {
                                    i.direction = direction.try_into()?;
                                }
                                i.map.name = mpc.map.clone();
                            }
                        }
                    }
                    Some(Data::ParseMap(map)) => {
                        self.player.map = maps::parse_map(&map.data, ctx)?;
                    }
                    Some(Data::WeaponData(data)) => {
                        self.automatic = data.automatic;
                    }
                    Some(Data::Say(say)) => {
                        ctx.speaker.speak(say.text.clone(), false)?;
                    }
                    Some(Data::Play(play)) => {
                        let sound_path = format!("sounds/{}", play.sound);
                        if !std::path::Path::new(&sound_path).exists() {
                            warn!("Tried to play a sound that does not exist, {}", play.sound);
                            return Ok(Transition::None);
                        }
                        let x = play.x.unwrap_or(self.player.x.try_into()?);
                        let y = play.y.unwrap_or(self.player.y.try_into()?);
                        if play.who != self.player.name && self.player.map.name == play.map {
                            ctx.sound
                                .play_3d(&sound_path, x.try_into()?, y.try_into()?, false);
                        } else {
                            if play.self_play.is_some() && play.who == self.player.name {
                                ctx.sound.play(&format!("sounds/{}", play.sound), false);
                            }
                        }
                    }
                    Some(Data::Buffer(mut buffer)) => {
                        if buffer.name == "" {
                            buffer.name = "misc".to_string();
                        }
                        if buffer.sound != "" {
                            ctx.sound.play(&format!("sounds/{}", buffer.sound), false);
                        }
                        self.buffer.add_item(buffer.text, buffer.name, true, ctx)?;
                    }
                    Some(Data::Pong(_)) => {
                        if self.pinging {
                            if let Some(timer) = self.ping_timer {
                                ctx.speaker.speak(
                                    format!(
                                        "The ping took {} milliseconds",
                                        timer.elapsed().as_millis()
                                    ),
                                    true,
                                )?;
                            }
                            self.ping_timer = None;
                            self.pinging = false;
                        }
                    }
                    Some(Data::Chat(chat)) => {
                        ctx.sound.play("sounds/notifications/chat.mp3", false);
                        self.buffer
                            .add_item(chat.message, "chat".to_string(), true, ctx)?;
                    }
                    Some(Data::Offline(off)) => {
                        ctx.sound.play("sounds/notifications/offline.mp3", false);
                        self.buffer.add_item(
                            format!("{} just went offline", off.who),
                            "connections".to_string(),
                            true,
                            ctx,
                        )?;
                        for i in 0..self.players.len() {
                            if self.players[i].name == off.who {
                                self.players.remove(i);
                                return Ok(Transition::None);
                            }
                        }
                    }
                    Some(Data::Online(on)) => {
                        ctx.sound.play("sounds/notifications/online.mp3", false);
                        self.buffer.add_item(
                            format!("{} just came online", on.who),
                            "connections".to_string(),
                            true,
                            ctx,
                        )?;
                        if on.who == self.player.name {
                            self.player.x = on.x.try_into()?;
                            self.player.y = on.y.try_into()?;
                            self.player.direction = on.direction.try_into()?;
                            self.player.map.name = on.map;
                        } else {
                            let mut player = Player::new(on.who, false, false);
                            player.x = on.x.try_into()?;
                            player.y = on.y.try_into()?;
                            player.direction = on.direction.try_into()?;
                            player.map = Grid::default();
                            player.map.name = on.map;
                            self.players.push(player);
                        }
                    }
                    _ => (),
                }
            }
        }
        Ok(Transition::None)
    }
    pub fn get_track(&self) -> Menu<()> {
        let players = self.player.sort(&self.players);
        let mut m = MenuBuilder::new("Track whom?", ());
        for i in &players {
            m = m.item(
                MenuItem::new(format!("{}", i.name)).on_activate(|mi, ctx, _| {
                    ctx.config
                        .data
                        .insert(String::from("tracking"), mi.text.clone());
                    ctx.speaker.speak(format!("Tracking {}", mi.text), true)?;
                    Ok(Transition::Pop(1))
                }),
            );
        }
        m.build()
    }
    pub fn tell_where(&self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if let Some(who) = ctx.config.data.get("tracking") {
            for i in &self.players {
                if i.name == who.clone() {
                    if i.name != self.player.name && i.map.name == self.player.map.name {
                        ctx.speaker.speak(distance(i, &self.player), true)?;
                        return Ok(());
                    } else {
                        ctx.speaker.speak("That player isn't in your map!", true)?;
                        return Ok(());
                    }
                }
            }
        } else {
            ctx.speaker.speak("You're not tracking anyone!", true)?;
            return Ok(());
        }
        ctx.speaker
            .speak("Error, Couldn't find the tracked player", true)?;
        Ok(())
    }
    pub fn update_players(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        self.player.update(ctx, true)?;
        for i in &mut self.players {
            if i.map.name == self.player.map.name {
                i.update(ctx, false)?;
            }
        }
        Ok(())
    }
    pub fn game_loop(&mut self, ctx: &mut GameContext) -> anyhow::Result<Transition<GameContext>> {
        if ctx.input.key_pressed(VirtualKeyCode::LControl)
            || ctx.input.key_pressed(VirtualKeyCode::RControl)
        {
            ctx.speaker.stop()?;
        }
        if ctx.input.key_pressed(VirtualKeyCode::P) && ctx.input.held_control() {
            ctx.stream = None;
            let i = Input::new()
                .set_title("Enter the URL to stream".to_string())
                .set_callback(|ctx, url| {
                    if url == "" {
                        ctx.speaker.speak("Canceled", true)?;
                        return Ok(Transition::Pop(1));
                    }
                    ctx.stream = Some(ctx.sound.play_url(&url, true, false));
                    if let Some(stream) = ctx.stream.as_mut() {
                        if let Some(volume) = ctx.config.data.get(&String::from("stream_volume")) {
                            stream.src.gain().set(volume.parse::<f64>()?)?;
                        }
                    }
                    Ok(Transition::Pop(1))
                });
            return Ok(Transition::Push(Box::new(i)));
        }
        if ctx.input.key_pressed(VirtualKeyCode::Return) {
            if ctx.input.held_shift() {
                let use_item = packets::UseItem::default();
                if let Some(client) = &mut ctx.client {
                    client.send(Data::UseItem(use_item))?;
                }
            } else {
                self.player
                    .map
                    .check_teleport(self.player.x, self.player.y, ctx)?;
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::H) {
            let hp = packets::Health::default();
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::Health(hp))?;
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::A) {
            let ammo = packets::Ammo::default();
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::Ammo(ammo))?;
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::Key1) {
            let mut draw = packets::Draw::default();
            draw.weapon = "pistol".to_string();
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::Draw(draw))?;
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::Key2) {
            let mut draw = packets::Draw::default();
            draw.weapon = "machinegun".to_string();
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::Draw(draw))?;
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::Key3) {
            let mut draw = packets::Draw::default();
            draw.weapon = "grenade_launcher".to_string();
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::Draw(draw))?;
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::R) {
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::Reload(packets::Reload::default()))?;
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::Space) {
            if self.automatic {
                self.firing = true;
            }
            let fire = packets::Fire::default();
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::Fire(fire))?;
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::T) {
            let m = self.get_track();
            if m.is_empty() {
                ctx.speaker.speak("You are alone on this map!", true)?;
                return Ok(Transition::None);
            }
            return Ok(Transition::Push(Box::new(m)));
        }
        if ctx.input.key_pressed(VirtualKeyCode::W) {
            self.tell_where(ctx)?;
        }
        if self.player.is_admin() {
            if ctx.input.key_pressed(VirtualKeyCode::K) {
                let i = Input::new()
                    .set_title("Enter your admin chat message".to_string())
                    .set_callback(|ctx, message| {
                        if message.is_empty() {
                            ctx.speaker.speak("Canceled", true)?;
                            return Ok(Transition::Pop(1));
                        }
                        let mut chat = packets::Chat::default();
                        chat.message = format!("/at {}", message);
                        if let Some(client) = ctx.client.as_mut() {
                            client.send(Data::Chat(chat))?;
                        }
                        Ok(Transition::Pop(1))
                    });
                return Ok(Transition::Push(Box::new(i)));
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::C) {
            if ctx.input.held_control() {
                let mut context: ClipboardContext = ClipboardProvider::new().unwrap();
                context.set_contents(self.buffer.get_item().get()).unwrap();
                ctx.speaker.speak("Text copyed", true)?;
            } else {
                let mut tile = String::from("the air");
                if let Some(tile2) = self.player.current_tile() {
                    tile = tile2;
                }
                ctx.speaker.speak(
                    format!(
                        "{}, {}, on {} on {}",
                        self.player.x, self.player.y, tile, self.player.map.name
                    ),
                    true,
                )?;
            }
        }
        if ctx.input.held_alt() {
            if ctx.input.key_pressed(VirtualKeyCode::Down) {
                if self.player.direction == 3 {
                    return Ok(Transition::None);
                }
                ctx.sound.play("sounds/player/turn.mp3", false);
                self.player.direction = 3;
                let mut mpc = packets::Move::default();
                mpc.direction = Some(3);
                if let Some(client) = &mut ctx.client {
                    client.send(Data::Move(mpc))?;
                }
            }
            if ctx.input.key_pressed(VirtualKeyCode::Right) {
                if self.player.direction == 0 {
                    return Ok(Transition::None);
                }
                ctx.sound.play("sounds/player/turn.mp3", false);
                self.player.direction = 0;
                let mut mpc = packets::Move::default();
                mpc.direction = Some(0);
                if let Some(client) = &mut ctx.client {
                    client.send(Data::Move(mpc))?;
                }
            }
            if ctx.input.key_pressed(VirtualKeyCode::Left) {
                if self.player.direction == 1 {
                    return Ok(Transition::None);
                }
                ctx.sound.play("sounds/player/turn.mp3", false);
                self.player.direction = 1;
                let mut mpc = packets::Move::default();
                mpc.direction = Some(1);
                if let Some(client) = &mut ctx.client {
                    client.send(Data::Move(mpc))?;
                }
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::Up) {
            if ctx.input.held_alt() {
                if self.player.direction == 2 {
                    return Ok(Transition::None);
                }
                ctx.sound.play("sounds/player/turn.mp3", false);
                self.player.direction = 2;
                let mut mpc = packets::Move::default();
                mpc.direction = Some(3);
                if let Some(client) = &mut ctx.client {
                    client.send(Data::Move(mpc))?;
                }
            } else {
                self.player.jump(ctx)?;
            }
        }
        if self.player.walk_timer.elapsed() >= self.player.walk_time {
            self.player.check_movement(ctx)?;
        }
        if ctx.input.key_pressed_os(VirtualKeyCode::F5) {
            if let Some(stream) = ctx.stream.as_mut() {
                if stream.src.gain().get()? > 0.0 {
                    let value = stream.src.gain().get()?;
                    stream.src.gain().set(value - 0.1)?;
                    ctx.config.data.insert(
                        "stream_volume".to_string(),
                        stream.src.gain().get()?.to_string(),
                    );
                }
            }
        }

        if ctx.input.key_pressed_os(VirtualKeyCode::F6) {
            if let Some(stream) = ctx.stream.as_mut() {
                if stream.src.gain().get()? < 1.0 {
                    let value = stream.src.gain().get()?;
                    stream.src.gain().set(value + 0.1)?;
                    ctx.config.data.insert(
                        "stream_volume".to_string(),
                        stream.src.gain().get()?.to_string(),
                    );
                }
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::F4) {
            if ctx.config.data.get("beacon").is_none() {
                ctx.speaker.speak("Beacons off", true)?;
                ctx.config
                    .data
                    .insert(String::from("beacon"), String::new());
            } else {
                ctx.speaker.speak("Beacons on", true)?;
                ctx.config.data.remove("beacon");
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::Tab) {
            let mut cycle = packets::Cycle::default();
            if ctx.input.held_shift() {
                cycle.direction = 0;
            } else {
                cycle.direction = 1;
            }
            if let Some(client) = &mut ctx.client {
                client.send(Data::Cycle(cycle))?;
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::F1) {
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::ServerStats(packets::ServerStats::default()))?;
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::F2) {
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::ServerNote(packets::ServerNote::default()))?;
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::F3) && !self.pinging {
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::Ping(packets::Ping::default()))?;
            }
            self.pinging = true;
            self.ping_timer = Some(Instant::now());
            ctx.speaker.speak("Pinging", true)?;
        }
        if ctx.input.key_pressed(VirtualKeyCode::Escape) {
            let m = MenuBuilder::new("Are you sure you want to exit?", ())
                .item(MenuItem::new("Yes").on_activate(|_, ctx, _| {
                    if let Some(client) = ctx.client.as_mut() {
                        client.send(Data::Close(packets::Close::default()))?;
                    }
                    let game_exit = GameExit::new();
                    Ok(Transition::Replace(1, Box::new(game_exit)))
                }))
                .item(MenuItem::new("No").on_activate(|_, ctx, _| {
                    ctx.speaker.speak("Canceled.", true)?;
                    return Ok(Transition::Pop(1));
                }))
                .build();
            return Ok(Transition::Push(Box::new(m)));
        }
        if ctx.input.key_pressed(VirtualKeyCode::Slash) {
            if ctx.input.held_control() {
                if let Some(client) = ctx.client.as_mut() {
                    client.send(Data::Who(packets::Who::default()))?;
                }
            } else {
                let i = Input::new()
                    .set_title("Enter your chat message!".to_string())
                    .set_callback(|ctx, msg| {
                        if msg == "" {
                            ctx.speaker.speak("Canceled!", true)?;
                            return Ok(Transition::Pop(1));
                        }
                        if let Some(client) = ctx.client.as_mut() {
                            let mut chat = packets::Chat::default();
                            chat.message = msg;
                            client.send(Data::Chat(chat))?;
                            return Ok(Transition::Pop(1));
                        }
                        Ok(Transition::None)
                    });
                return Ok(Transition::Push(Box::new(i)));
            }
        }
        if ctx.input.key_pressed(VirtualKeyCode::Comma) {
            if ctx.input.held_shift() {
                self.buffer.get_item().first(ctx)?;
            } else {
                self.buffer.get_item().previous(ctx)?;
            }
        } else if ctx.input.key_pressed(VirtualKeyCode::Period) {
            if ctx.input.held_shift() {
                self.buffer.get_item().last(ctx)?;
            } else {
                self.buffer.get_item().next(ctx)?;
            }
        } else if ctx.input.key_pressed(VirtualKeyCode::LBracket) {
            if ctx.input.held_shift() {
                self.buffer.first(ctx)?;
            } else {
                self.buffer.previous(ctx)?;
            }
        } else if ctx.input.key_pressed(VirtualKeyCode::RBracket) {
            if ctx.input.held_shift() {
                self.buffer.last(ctx)?;
            } else {
                self.buffer.next(ctx)?;
            }
        }
        Ok(Transition::None)
    }
    pub fn spawn_player(&mut self, name: String, admin: bool, dev: bool) {
        self.player = Player::new(name, admin, dev);
        self.player.map = Grid::new(2000, 100, String::from("main"));
        self.player.map.add(0, 2000, 0, 0, "concrete2");
        self.player.map.add(50, 56, 3, 4, "wall_metal");
        self.player.map.add(20, 30, 0, 15, "grass2");
    }
}

impl State<GameContext> for Game {
    fn on_push(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        ctx.speaker.speak("Connecting...", true)?;
        ctx.connect()?;
        if let Some(client) = ctx.client.as_mut() {
            client.send(Data::Connect(packets::Connect::default()))?;
        }
        Ok(())
    }
    fn on_pop(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        ctx.client = None;
        ctx.stream = None;
        Ok(())
    }
    fn on_update(
        &mut self,
        ctx: &mut GameContext,
        depth: usize,
    ) -> anyhow::Result<Transition<GameContext>> {
        let res = self.process_packet(ctx)?;
        match res {
            Transition::None => (),
            _ => {
                return Ok(res);
            }
        }
        self.player.gravity_check(ctx)?;
        self.update_players(ctx)?;
        ctx.sound
            .set_position((self.player.x as f64, self.player.y as f64, 0.0))?;
        self.player
            .map
            .grid_loop(self.player.x.try_into()?, self.player.y.try_into()?, ctx)?;
        if self.firing && ctx.input.key_released(VirtualKeyCode::Space) || self.firing && depth != 0
        {
            if let Some(client) = ctx.client.as_mut() {
                client.send(Data::FireStop(packets::FireStop::default()))?;
            }
        }
        if let Some(client) = ctx.client.as_mut() {
            client.poll()?;
            if depth != 0 {
                return Ok(Transition::None);
            }
            match client.state {
                NetworkState::Connected => return self.game_loop(ctx),
                NetworkState::RawConnection => match self.state {
                    GameState::Creating => {
                        return self.create(ctx);
                    }
                    GameState::LoggingIn => {
                        return self.login(ctx);
                    }
                    GameState::SettingAccount => {
                        return self.set_account(ctx);
                    }
                    _ => (),
                },
                _ => {
                    if depth == 0 && ctx.input.key_pressed(VirtualKeyCode::Escape) {
                        return Ok(Transition::Pop(1));
                    }
                }
            }
        }
        Ok(Transition::None)
    }
}
