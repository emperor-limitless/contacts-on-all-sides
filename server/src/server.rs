use crate::{
    bans, connection::Connection, dm::Dm, maps, maps::grid::Grid, player::Player,
    readable_time::format_time, weapon::Weapon,
};
use enet::*;
use fernet::Fernet;
use once_cell::sync::Lazy;
use prost::Message;
use rand::{rngs::ThreadRng, Rng};
use semver::Version;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::Read, path::Path, sync::Mutex, time::Instant};
pub mod packets {
    include!(concat!(env!("OUT_DIR"), "/network.packets.rs"));
}
use packets::packet::Data;

const RULES: &str = include_str!("rules.txt");

pub static mut SERVER: Lazy<Mutex<Server>> = Lazy::new(|| {
    let mut server = Mutex::new(Server::new(18832).unwrap());
    server.get_mut().unwrap().load().unwrap();
    server
});

pub fn get_server() -> &'static mut Server {
    unsafe { SERVER.get_mut().unwrap() }
}
#[derive(Serialize, Deserialize, Default)]
pub struct ServerData {
    peak: usize,
    #[serde(default)]
    bans: bans::Bans,
}
pub struct Server {
    #[allow(dead_code)]
    enet: Enet,
    host: Host<()>,
    #[allow(dead_code)]
    addr: Address,
    up_timer: Instant,
    pub data: ServerData,
    pub rng: ThreadRng,
    pub players: HashMap<Peer, Player>,
    pub weapons: Vec<Weapon>,
    foo: Dm,
    pub connections: HashMap<Peer, Connection>,
    pub maps: HashMap<String, Grid>,
}

impl Server {
    pub fn new(port: u16) -> anyhow::Result<Self> {
        let enet = Enet::new()?;
        let addr = Address::new(std::net::Ipv4Addr::UNSPECIFIED, port);
        let host = enet.create_host::<()>(
            Some(&addr),
            100,
            ChannelLimit::Maximum,
            BandwidthLimit::Unlimited,
            BandwidthLimit::Unlimited,
        )?;
        let maps = maps::parse_all_maps()?;
        Ok(Self {
            enet,
            host,
            addr,
            up_timer: Instant::now(),
            data: ServerData::default(),
            rng: rand::thread_rng(),
            weapons: vec![],
            foo: Dm::new(
                String::new(),
                String::from("KcBkgRw_ju2TbjHc9V21VY9-bm0U2mRAKPZdM9aKZ_E="),
            ),
            players: HashMap::new(),
            connections: HashMap::new(),
            maps,
        })
    }
    pub fn load(&mut self) -> anyhow::Result<()> {
        let data_file = Path::new("server.dat");
        if !data_file.exists() {
            return Ok(());
        }
        let f = match Fernet::new("gcj57_LgWAe8KnRUqGZLf7RnfxQWs7ZzPeAkCLHZh5M=") {
            Some(f) => f,
            None => return Ok(()),
        };
        let mut file = fs::File::open(data_file)?;
        let mut text = String::new();
        file.read_to_string(&mut text)?;
        text = String::from_utf8(f.decrypt(&text)?)?;
        self.data = serde_json::from_str(&text)?;
        Ok(())
    }
    pub fn save(&self) -> anyhow::Result<()> {
        let data_file = Path::new("server.dat");
        let f = match Fernet::new("gcj57_LgWAe8KnRUqGZLf7RnfxQWs7ZzPeAkCLHZh5M=") {
            Some(f) => f,
            None => return Ok(()),
        };
        let data = serde_json::to_string(&self.data)?;
        let text = f.encrypt(data.as_bytes());
        fs::write(data_file, text)?;
        Ok(())
    }
    pub fn get_player(&mut self, peer: &Peer) -> Option<&mut Player> {
        self.players.get_mut(peer)
    }
    pub fn get_player_by_name(&mut self, name: &str) -> Option<&mut Player> {
        if let Some((_, player)) = self
            .players
            .iter_mut()
            .find(|(_, i)| i.name.to_lowercase() == name.to_lowercase())
        {
            return Some(player);
        }
        None
    }
    pub fn get_map(&mut self, name: &str) -> Option<&mut Grid> {
        self.maps.get_mut(name)
    }
    pub fn get_connection(&mut self, peer: Peer) -> Option<&mut Connection> {
        self.connections.get_mut(&peer)
    }
    pub fn play(&mut self, sound: &str, x: isize, y: isize, map: &str) -> anyhow::Result<()> {
        let mut play = packets::Play::default();
        play.x = Some(x.try_into()?);
        play.y = Some(y.try_into()?);
        play.map = map.to_string();
        play.sound = sound.to_string();
        let data = Data::Play(play);
        for (_, i) in &mut self.players {
            if i.data.map == map {
                get_server().send(i.addr, data.clone())?;
            }
        }
        Ok(())
    }
    pub fn send(&mut self, mut peer: Peer, data: Data) -> anyhow::Result<()> {
        let mut pac = packets::Packet::default();
        pac.data = Some(data);
        let mut buf = vec![];
        pac.encode(&mut buf)?;
        peer.send_packet(
            Packet::new(
                self.foo.encrypto(buf.as_slice()).as_bytes(),
                PacketMode::ReliableSequenced,
            )?,
            0,
        )?;
        Ok(())
    }
    pub fn broadcast(&mut self, data: Data) -> anyhow::Result<()> {
        let mut pac = packets::Packet::default();
        pac.data = Some(data);
        let mut buf = vec![];
        pac.encode(&mut buf)?;
        for (peer, _) in self.players.iter() {
            peer.clone().send_packet(
                Packet::new(
                    self.foo.encrypto(buf.as_slice()).as_bytes(),
                    PacketMode::ReliableSequenced,
                )?,
                0,
            )?;
        }
        Ok(())
    }
    pub fn notify(&mut self, text: String) -> anyhow::Result<()> {
        let mut note = packets::Buffer::default();
        note.text = text;
        note.name = String::from("notifications");
        note.sound = String::from("notifications/alert.mp3");
        self.broadcast(Data::Buffer(note))?;
        Ok(())
    }
    pub fn admin_tell(&mut self, text: String) -> anyhow::Result<()> {
        let mut buf = packets::Buffer::default();
        buf.text = text;
        buf.name = String::from("admin alerts");
        buf.sound = String::from("notifications/admin_tell.mp3");
        for (_, i) in &self.players {
            if i.admin() {
                get_server().send(i.addr, Data::Buffer(buf.clone()))?;
            }
        }
        Ok(())
    }
    pub fn process_command(&mut self, command: &str, peer: Peer) -> anyhow::Result<()> {
        let parsed = command.split_whitespace().collect::<Vec<&str>>();
        if parsed.is_empty() {
            return Ok(());
        }
        if parsed[0] == "rawmap" {
            if let Some(player) = self.get_player(&peer) {
                if !player.admin() {
                    return Ok(());
                }
                let mut buf = packets::Buffer::default();
                buf.text = std::fs::read_to_string(format!("maps/{}.map", player.data.map))?;
                self.send(peer, Data::Buffer(buf))?;
            }
        } else if parsed[0] == "rawdata" && parsed.len() > 2 {
            if let Some(player) = self.get_player(&peer) {
                if !player.admin() {
                    return Ok(());
                }
                std::fs::write(
                    format!("maps/{}.map", player.data.map),
                    command.replacen("rawdata ", "", 1),
                )?;
                get_server().maps.insert(
                    player.data.map.clone(),
                    maps::parse_map(&format!("maps/{}.map", player.data.map.clone()))?,
                );
                let parse = packets::ParseMap {
                    data: std::fs::read_to_string(format!("maps/{}.map", player.data.map.clone()))?,
                };
                for (_, i) in &get_server().players {
                    if i.data.map == player.data.map {
                        get_server().send(i.addr, Data::ParseMap(parse.clone()))?;
                        get_server().send(
                            i.addr,
                            Data::Buffer(packets::Buffer {
                                text: String::from("Map updated!"),
                                name: String::new(),
                                sound: String::new(),
                            }),
                        )?;
                    }
                }
            }
        } else if parsed[0] == "admin" && parsed.len() == 2 {
            if let Some(player) = self.get_player(&peer) {
                if player.data.dev {
                    if let Some(handle) = get_server().get_player_by_name(parsed[1]) {
                        if handle.data.admin {
                            handle.data.admin = false;
                            let mut buf = packets::Buffer::default();
                            buf.text = String::from("You are no longer an admin");
                            get_server().send(handle.addr, Data::Buffer(buf))?;
                            get_server().notify(format!(
                                "{} Has been removed from the administrator status",
                                handle.name
                            ))?;
                        } else {
                            handle.data.admin = true;
                            let mut buf = packets::Buffer::default();
                            buf.text = String::from("You are now an admin");
                            get_server().send(handle.addr, Data::Buffer(buf))?;
                            get_server().notify(format!(
                                "{} Has been promoted to administrator",
                                handle.name
                            ))?;
                        }
                    }
                }
            }
        } else if parsed[0] == "at" {
            if let Some(player) = self.get_player(&peer) {
                if player.admin() {
                    get_server().admin_tell(format!(
                        "Admin chat from {}: {}",
                        player.name,
                        command.replacen("at ", "", 1)
                    ))?;
                }
            }
        } else if parsed[0] == "admintell" {
            if let Some(player) = self.get_player(&peer) {
                get_server().admin_tell(format!(
                    "Admin tell from {}: {}",
                    player.name,
                    command.replacen("admintell ", "", 1)
                ))?;
            }
        } else if parsed[0] == "notify" && parsed.len() > 2 {
            if let Some(player) = self.get_player(&peer) {
                if player.admin() {
                    get_server().notify(command.replacen("notify ", "", 1))?;
                    get_server().admin_tell(format!(
                        "{} Has just sent a notification to the server!",
                        player.name
                    ))?;
                }
            }
        } else if parsed[0] == "me" && parsed.len() > 1 {
            if let Some(player) = self.get_player(&peer) {
                let mut buf = packets::Buffer::default();
                buf.text = format!("{} {}", player.name, command.replacen("me ", "", 1));
                buf.name = String::from("chat");
                buf.sound = String::from("notifications/chat.mp3");
                self.send(peer, Data::Buffer(buf))?;
            }
        } else if parsed[0] == "can_chat" && parsed.len() > 1 {
            if let Some(player) = self.get_player(&peer) {
                if !player.admin() {
                    return Ok(());
                }
                if let Some(handle) = get_server().get_player_by_name(parsed[1]) {
                    if handle.data.can_chat {
                        handle.data.can_chat = false;
                        get_server().notify(format!(
                            "{}'s chats were disabled by {}",
                            handle.name, player.name
                        ))?;
                    } else {
                        handle.data.can_chat = true;
                        get_server().notify(format!(
                            "{}'s chats were enabled by {}",
                            handle.name, player.name
                        ))?;
                    }
                }
            }
        } else if parsed[0] == "rules" {
            let mut buf = packets::Buffer::default();
            buf.text = RULES.to_string();
            self.send(peer, Data::Buffer(buf))?;
        } else if parsed[0] == "agree" {
            if let Some(player) = self.get_player(&peer) {
                if !player.data.agreed_to_rules {
                    player.data.agreed_to_rules = true;
                    player.say(String::from(
                        "Success, You have agreed to the rules, Welcome to the game.",
                    ))?;
                } else {
                    player.say(String::from("You have already agreed to the rules!"))?;
                }
            }
        } else if parsed[0] == "save" {
            if let Some(player) = self.get_player(&peer) {
                if player.admin() {
                    for i in get_server().players.values_mut() {
                        i.save()?;
                        get_server().send(i.addr, Data::Close(packets::Close::default()))?;
                    }
                    println!("Server have been restarted by {}", player.name);
                    self.save()?;
                    std::process::exit(0);
                }
            }
        } else if parsed[0] == "hit_ping" {
            if let Some(player) = self.get_player(&peer) {
                if player.data.hit_ping {
                    player.data.hit_ping = false;
                    player.say("Hitting ping turned off!".to_string())?;
                } else {
                    player.data.hit_ping = true;
                    player.say("Hitting ping turned on!".to_string())?;
                }
            }
        } else if parsed[0] == "kick" && parsed.len() == 2 {
            if let Some(player) = get_server().get_player(&peer) {
                if player.admin() {
                    if let Some(handle) = self.get_player_by_name(parsed[1]) {
                        get_server().notify(format!(
                            "{} have been kicked by {}",
                            handle.name, player.name
                        ))?;
                        get_server().send(handle.addr, Data::Close(packets::Close::default()))?;
                        let mut off = packets::Offline::default();
                        off.who = handle.name.clone();
                        get_server().broadcast(Data::Offline(off))?;
                        handle.save()?;
                        get_server().players.remove(&handle.addr);
                    }
                }
            }
        } else if parsed[0] == "ban" && parsed.len() > 1 {
            if let Some(player) = get_server().get_player(&peer) {
                if !player.admin() {
                    return Ok(());
                }
                if let Some(handle) = get_server().get_player_by_name(parsed[1]) {
                    get_server()
                        .data
                        .bans
                        .add_ban(handle.name.clone(), handle.data.id.clone());
                    get_server().save()?;
                    get_server().notify(format!(
                        "{} Have been banned by {}",
                        handle.name, player.name
                    ))?;
                    get_server().send(handle.addr, Data::Close(packets::Close::default()))?;
                    let mut off = packets::Offline::default();
                    off.who = handle.name.clone();
                    get_server().broadcast(Data::Offline(off))?;
                    handle.save()?;
                    get_server().players.remove(&handle.addr);
                } else {
                    if Path::new(format!("players/{}", parsed[1]).as_str()).exists() {
                        if let Some(peer) = self.host.peers().last() {
                            if let Some(player2) = Player::load(parsed[1].to_string(), peer)? {
                                if get_server().data.bans.get_user(player2.data.id.clone()) {
                                    get_server().data.bans.remove_ban(player2.name.clone());
                                    get_server().save()?;
                                    get_server().notify(format!(
                                        "{} Have been unbanned by {}",
                                        player2.name, player.name
                                    ))?;
                                    return Ok(());
                                }
                                get_server()
                                    .data
                                    .bans
                                    .add_ban(player2.name.clone(), player2.data.id);
                                get_server().save()?;
                                get_server().notify(format!(
                                    "{} Have been banned by {}",
                                    player2.name, player.name
                                ))?;
                            }
                        }
                    } else {
                        player.say("Error, Player not found!".to_string())?;
                        return Ok(());
                    }
                }
            }
        } else if parsed[0] == "timed_ban" && parsed.len() > 2 {
            if let Some(player) = get_server().get_player(&peer) {
                if !player.admin() {
                    return Ok(());
                }
                let time = match parsed[2].parse::<u64>() {
                    Ok(time) => time,
                    Err(e) => {
                        player.say(format!("Error in second argument(Time), Reason: {}", e))?;
                        return Ok(());
                    }
                };
                if let Some(handle) = get_server().get_player_by_name(parsed[1]) {
                    get_server().data.bans.add_temporary(
                        handle.name.clone(),
                        handle.data.id.clone(),
                        time * 1000 * 60,
                    );
                    get_server().save()?;
                    get_server().notify(format!(
                        "{} Have been temporarily banned by {} For {}",
                        handle.name,
                        player.name,
                        format_time((time * 1000 * 60).try_into()?)
                    ))?;
                    get_server().send(handle.addr, Data::Close(packets::Close::default()))?;
                    let mut off = packets::Offline::default();
                    off.who = handle.name.clone();
                    get_server().broadcast(Data::Offline(off))?;
                    handle.save()?;
                    get_server().players.remove(&handle.addr);
                } else {
                    if Path::new(format!("players/{}", parsed[1]).as_str()).exists() {
                        if let Some(peer) = self.host.peers().last() {
                            if let Some(player2) = Player::load(parsed[1].to_string(), peer)? {
                                if get_server()
                                    .data
                                    .bans
                                    .get_temporary(player2.data.id.clone())
                                {
                                    get_server()
                                        .data
                                        .bans
                                        .remove_temporary(player2.name.clone());
                                    get_server().save()?;
                                    get_server().notify(format!(
                                        "{} Have been unbanned by {}",
                                        player2.name, player.name
                                    ))?;
                                    return Ok(());
                                }
                                get_server().data.bans.add_temporary(
                                    player2.name.clone(),
                                    player2.data.id,
                                    time * 1000 * 60,
                                );
                                get_server().save()?;
                                get_server().notify(format!(
                                    "{} Have been temporarily banned by {} For {}",
                                    player2.name,
                                    player.name,
                                    format_time((time * 1000 * 60).try_into()?)
                                ))?;
                            }
                        }
                    } else {
                        player.say("Error, Player not found!".to_string())?;
                        return Ok(());
                    }
                }
            }
        } else if parsed[0] == "move" && parsed.len() >= 4 {
            if let Some(player) = self.get_player(&peer) {
                if !player.admin() {
                    return Ok(());
                }
                if let Some(handle) = get_server().get_player_by_name(parsed[1]) {
                    let mut map = handle.data.map.clone();
                    if parsed.len() == 5 {
                        map = parsed[4].to_string();
                    }
                    if !get_server().maps.contains_key(&map) {
                        player.say("That map does not exist!".to_string())?;
                        return Ok(());
                    }
                    let x = match parsed[2].parse() {
                        Ok(x) => x,
                        Err(e) => {
                            player.say(format!(
                                "Error: Couldn't change the player's location, Reason: {}",
                                e
                            ))?;
                            return Ok(());
                        }
                    };
                    let y = match parsed[3].parse() {
                        Ok(y) => y,
                        Err(e) => {
                            player.say(format!(
                                "Error: Couldn't change the player's location, Reason: {}",
                                e
                            ))?;
                            return Ok(());
                        }
                    };
                    match handle.change_map(x, y, map) {
                        Ok(_) => (),
                        Err(e) => {
                            player.say(format!(
                                "Error: Couldn't change the player's location, Reason: {}",
                                e
                            ))?;
                        }
                    }
                }
            }
        } else if parsed[0] == "newmap" && parsed.len() >= 5 {
            if let Some(player) = self.get_player(&peer) {
                if !player.admin() {
                    return Ok(());
                }
                if get_server().get_map(parsed[1]).is_some() {
                    player.say("That map already exists!".to_string())?;
                    return Ok(());
                }
                let text = format!(
                    "map {}
                maxx {}
                maxy {}
                tile 0 {} 0 0 {}",
                    parsed[1], parsed[2], parsed[3], parsed[2], parsed[4]
                );
                match fs::write(format!("maps/{}.map", parsed[1]), text) {
                    Ok(_) => {
                        player.change_map(0, 0, parsed[1].to_string())?;
                        get_server().admin_tell(format!(
                            "{} Has created a new map: {}",
                            player.name, parsed[1]
                        ))?;
                        self.maps.insert(
                            parsed[1].to_string(),
                            maps::parse_map(&format!("maps/{}.map", parsed[1]))?,
                        );
                    }
                    Err(e) => {
                        player.say(format!("Error: Unable to create the map, Reason: {}", e))?;
                    }
                }
            }
        } else if parsed[0] == "remmap" {
            if let Some(player) = self.get_player(&peer) {
                let mut map = player.data.map.clone();
                if parsed.len() == 2 {
                    map = parsed[1].to_string();
                }
                if map == "main" || map == "safe_zone" {
                    player.say("Error: This map cannot be deleted".to_string())?;
                    return Ok(());
                }
                let map_path = format!("maps/{}.map", map);
                let path = Path::new(&map_path);
                if !path.exists() {
                    player.say("That map does not exist".to_string())?;
                    return Ok(());
                }
                match fs::remove_file(path) {
                    Ok(_) => {
                        player.say("Success".to_string())?;
                        get_server()
                            .admin_tell(format!("{} Have deleted the map {}", player.name, map))?;
                        player.change_map(0, 0, "main".to_string())?;
                    }
                    Err(e) => {
                        player.say(format!("Error: Unable to delete the map, Reason: {}", e))?;
                    }
                }
            }
        } else if parsed[0] == "give" && parsed.len() >= 4 {
            if let Some(player) = self.get_player(&peer) {
                if !player.admin() {
                    return Ok(());
                }
                if let Some(handle) = get_server().get_player_by_name(parsed[1]) {
                    let amount = match parsed[3].parse() {
                        Ok(amount) => amount,
                        Err(e) => {
                            player.say(format!("Error: Unable to give, Reason: {}", e))?;
                            return Ok(());
                        }
                    };
                    handle.give(parsed[2], amount)?;
                    get_server().admin_tell(format!(
                        "{} has just given {} {} {}s",
                        player.name, handle.name, parsed[3], parsed[2]
                    ))?;
                }
            }
        } else if parsed[0] == "giveall" && parsed.len() >= 3 {
            if let Some(player) = self.get_player(&peer) {
                if !player.admin() {
                    return Ok(());
                }
                let amount = match parsed[2].parse() {
                    Ok(amount) => amount,
                    Err(e) => {
                        player.say(format!("Error: Unable to give, Reason: {}", e))?;
                        return Ok(());
                    }
                };
                for (_, i) in &mut get_server().players {
                    i.give(parsed[1], amount)?;
                }
                get_server().admin_tell(format!(
                    "{} has just given everyone {} {}s",
                    player.name, parsed[2], parsed[1]
                ))?;
            }
        } else if parsed[0] == "note" && parsed.len() > 1 {
            if let Some(player) = self.get_player(&peer) {
                if player.admin() {
                    let cmd = command.replacen("note ", "", 1);
                    fs::write("note.txt", &cmd)?;
                    get_server().notify(format!(
                        "{} Have just changed the server note to {}",
                        player.name, cmd
                    ))?;
                }
            }
        }
        Ok(())
    }
    pub fn process_packet(&mut self, buf: &[u8], addr: Peer) -> anyhow::Result<()> {
        let packet = packets::Packet::decode(buf);
        if packet.is_err() {
            return Ok(());
        }
        let pac = packet?;
        match pac.data {
            Some(Data::Login(mut login)) => {
                if self.get_player_by_name(&login.user).is_none() {
                    if Player::check_account(login.user.clone(), login.password)? {
                        if login.version == "" {
                            login.version = "0.1.0".to_string();
                        }
                        let version = Version::parse(&login.version)?;
                        let server_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
                        if version < server_version {
                            let error = packets::Error { reason: format!("A new version is available for download, Your version: {} Latest version: {}, Download the new version from https://arelius.org/coas/coas.zip, If you are running linux, Download from https://arelius.org/coas/coas_linux.zip", version, server_version) };
                            get_server().send(addr, Data::Error(error))?;
                            return Ok(());
                        }
                        let banned = self.data.bans.get_user(login.id.clone());
                        let temporary_banned = self.data.bans.get_temporary(login.id.clone());
                        if banned || temporary_banned {
                            let mut error = packets::Error {
                                reason: String::from("You have been banned..."),
                            };
                            if temporary_banned {
                                error.reason += &format!(
                                    "Time remaining: {}",
                                    format_time(
                                        self.data
                                            .bans
                                            .get_temporary_remaining(login.id.clone())
                                            .try_into()?
                                    )
                                )
                            }
                            get_server().send(addr, Data::Error(error))?;
                            return Ok(());
                        }
                        let mut players = vec![];
                        for (_, i) in &self.players {
                            let mut player = packets::Player::default();
                            player.x = i.data.x.try_into()?;
                            player.y = i.data.y.try_into()?;
                            player.direction = i.data.direction.try_into()?;
                            player.map = i.data.map.clone();
                            player.name = i.name.clone();
                            players.push(player);
                        }
                        let mut connected = packets::Connected::default();
                        connected.players = players;
                        self.add_player(&login.user, addr, login.id.clone())?;
                        if let Some(player) = self.get_player(&addr) {
                            if login.dev.is_some() {
                                if login.user == "emperor-limitless" {
                                    player.data.dev = true;
                                    connected.dev = Some(true);
                                }
                            }
                            if player.admin() {
                                connected.admin = Some(true);
                            }
                        }
                        self.send(addr, Data::Connected(connected))?;
                        if let Some(player) = get_server().get_player(&addr) {
                            let mut parse = packets::ParseMap::default();
                            parse.data =
                                std::fs::read_to_string(format!("maps/{}.map", player.data.map))?;
                            self.send(addr, Data::ParseMap(parse))?;
                            let mut online = packets::Online::default();
                            online.who = login.user;
                            online.x = player.data.x.try_into()?;
                            online.y = player.data.y.try_into()?;
                            online.map = player.data.map.clone();
                            online.direction = player.data.direction.try_into()?;
                            self.broadcast(Data::Online(online))?;
                        }
                    } else {
                        let mut error = packets::Error::default();
                        error.reason = "Invalid username or password!".to_string();
                        self.send(addr, Data::Error(error))?;
                        return Ok(());
                    }
                } else if self.get_player_by_name(&login.user).is_some() {
                    let mut error = packets::Error::default();
                    error.reason = "A player with the same name is already logged in!".to_string();
                    self.send(addr, Data::Error(error))?;
                    return Ok(());
                }
            }
            Some(Data::Move(mut mpc)) => {
                if let Some(player) = self.get_player(&addr) {
                    if let Some(x) = mpc.x {
                        player.data.x = x.try_into()?;
                    }
                    if let Some(y) = mpc.y {
                        player.data.y = y.try_into()?;
                    }
                    if let Some(direction) = mpc.direction {
                        player.data.direction = direction.try_into()?;
                    }
                    mpc.map = player.data.map.clone();
                    mpc.who = player.name.clone();
                    get_server().broadcast(Data::Move(mpc.clone()))?;
                    if mpc.silent.is_none() {
                        if let Some(map) = get_server().get_map(&player.data.map) {
                            if let Some(tile) = map.get_tile(player.data.x, player.data.y) {
                                let mut play = packets::Play::default();
                                play.x = Some(player.data.x.try_into()?);
                                play.y = Some(player.data.y.try_into()?);
                                play.who = player.name.clone();
                                play.sound = format!(
                                    "steps/{}/step{}.mp3",
                                    tile,
                                    get_server().rng.gen_range(1..=5)
                                );
                                play.map = player.data.map.clone();
                                self.broadcast(Data::Play(play))?;
                            }
                        }
                    }
                }
            }
            Some(Data::Play(mut play)) => {
                if let Some(player) = self.get_player(&addr) {
                    play.who = player.name.clone();
                    self.broadcast(Data::Play(play))?;
                }
            }
            Some(Data::ServerStats(_)) => {
                let mut buffer = packets::Buffer::default();
                buffer.text = format!(
                    "Server version {}, Up for {}, Peak: {}",
                    env!("CARGO_PKG_VERSION"),
                    format_time(self.up_timer.elapsed().as_millis()),
                    self.data.peak
                );
                self.send(addr, Data::Buffer(buffer))?;
            }
            Some(Data::ServerNote(_)) => {
                let note = match fs::read_to_string("note.txt") {
                    Ok(note) => note,
                    Err(_) => "No server note at the moment".to_string(),
                };
                let mut buffer = packets::Buffer::default();
                buffer.text = format!("Server note: {}", note);
                self.send(addr, Data::Buffer(buffer))?;
            }
            Some(Data::UseItem(_)) => {
                if let Some(player) = self.get_player(&addr) {
                    player.use_item()?;
                }
            }
            Some(Data::Cycle(cycle)) => {
                if let Some(player) = self.get_player(&addr) {
                    player.cycle(cycle.direction.try_into()?)?;
                }
            }
            Some(Data::Teleport(teleport)) => {
                if let Some(player) = self.get_player(&addr) {
                    player.change_map(
                        teleport.x.try_into()?,
                        teleport.y.try_into()?,
                        teleport.map.clone(),
                    )?;
                }
            }
            Some(Data::Draw(draw)) => {
                if let Some(player) = self.get_player(&addr) {
                    player.draw(draw.weapon.clone())?;
                }
            }
            Some(Data::Fire(_)) => {
                if let Some(player) = self.get_player(&addr) {
                    player.fire()?;
                }
            }
            Some(Data::Reload(_)) => {
                if let Some(player) = self.get_player(&addr) {
                    player.reload()?;
                }
            }
            Some(Data::FireStop(_)) => {
                if let Some(player) = self.get_player(&addr) {
                    player.stop_fire();
                }
            }
            Some(Data::Ammo(_)) => {
                if let Some(player) = self.get_player(&addr) {
                    let mut say = packets::Say::default();
                    if player.data.weapon.is_empty() {
                        say.text = "You don't have a weapon loaded".to_string();
                    } else {
                        say.text = format!(
                            "You have {} ammo loaded, And {} cartridges remaining!",
                            player.data.get_ammo(),
                            player.data.get_cartridges()
                        );
                    }
                    self.send(addr, Data::Say(say))?;
                }
            }
            Some(Data::Health(_)) => {
                if let Some(player) = self.get_player(&addr) {
                    player.say(format!("{}HP", player.data.health))?;
                }
            }
            Some(Data::Create(create)) => {
                let banned = self.data.bans.get_user(create.id.clone());
                let temporary_banned = self.data.bans.get_temporary(create.id.clone());
                if banned || temporary_banned {
                    let mut error = packets::Error {
                        reason: String::from("You have been banned..."),
                    };
                    if temporary_banned {
                        error.reason += &format!(
                            "Time remaining: {}",
                            format_time(
                                self.data
                                    .bans
                                    .get_temporary_remaining(create.id.clone())
                                    .try_into()?
                            )
                        )
                    }
                    get_server().send(addr, Data::Error(error))?;
                    return Ok(());
                }
                if Player::create(addr, create.user.clone(), create.password, create.email)? {
                    self.send(addr, Data::Created(packets::Created::default()))?;
                    self.admin_tell(format!("Alert: {} Has been created", create.user))?;
                } else {
                    let mut error = packets::Error::default();
                    error.reason = "A player with that name already exists!".to_string();
                    self.send(addr, Data::Error(error))?;
                    return Ok(());
                }
            }
            Some(Data::Close(_)) => {
                if let Some(player) = self.players.remove(&addr) {
                    if player.got_hit {
                        get_server().notify(format!(
                            "{} Have been detected to cheat and is banned for 3 minutes",
                            player.name
                        ))?;
                        get_server().data.bans.add_temporary(
                            player.name.clone(),
                            player.data.id.clone(),
                            180000,
                        );
                        get_server().save()?;
                    }
                    let mut offline = packets::Offline::default();
                    offline.who = player.name.clone();
                    self.broadcast(Data::Offline(offline))?;
                    player.save()?;
                }
            }
            Some(Data::Chat(chat)) => {
                if let Some(player) = self.get_player(&addr) {
                    if chat.message.starts_with("/") {
                        if let Some(msg) = chat.message.strip_prefix("/") {
                            self.process_command(msg, addr)?;
                        }
                    } else {
                        if !player.data.agreed_to_rules {
                            player.say(format!("You haven't agreed to the game rules yet. Please read the rules by typing /rules, And accept them by typing /agree, Otherwise, Please delete the game"))?;
                            return Ok(());
                        }
                        if player.data.can_chat {
                            let mut cht = packets::Chat::default();
                            cht.message = format!("{} says: {}", &player.name, chat.message);
                            self.broadcast(Data::Chat(cht))?;
                        } else {
                            player.say("You are not allowed to chat".to_string())?;
                        }
                    }
                }
            }
            Some(Data::Connect(_)) => {
                self.send(addr, Data::Connect(packets::Connect::default()))?;
            }
            Some(Data::Ping(_)) => {
                self.send(addr, Data::Pong(packets::Pong::default()))?;
            }
            Some(Data::Who(_)) => {
                let mut names = vec![];
                for (_, i) in self.players.iter() {
                    names.push(format!(
                        "{}, {} kills, {} deaths",
                        i.name, i.data.kills, i.data.deaths
                    ));
                }
                let mut buffer = packets::Buffer::default();
                if names.len() == 1 {
                    buffer.text = "You are alone!".to_string();
                } else {
                    buffer.text = format!(
                        "There are currently {} players online: {}",
                        names.len(),
                        names.join(", ")
                    );
                }
                self.send(addr, Data::Buffer(buffer))?;
            }
            _ => (),
        }
        Ok(())
    }
    pub fn add_player(&mut self, user: &str, addr: Peer, id: String) -> anyhow::Result<()> {
        let mut player = match Player::load(user.to_string(), addr)? {
            Some(player) => player,
            None => return Ok(()),
        };
        player.data.id = id;
        self.players.insert(addr, player);
        if self.players.len() > self.data.peak {
            self.data.peak = self.players.len();
            self.notify(format!(
                "We have reached a new peak of  {} players",
                self.data.peak
            ))?;
            self.save()?;
        }
        Ok(())
    }
    pub fn add_connection(&mut self, addr: Peer) {
        let connection = Connection::new(addr);
        self.connections.insert(addr, connection);
    }
    pub fn players_update(&mut self) -> anyhow::Result<()> {
        for (_, i) in self.players.iter_mut() {
            i.update()?;
        }
        Ok(())
    }
    pub fn connections_update(&mut self) -> anyhow::Result<()> {
        for (_, i) in self.connections.iter_mut() {
            i.update()?;
        }
        Ok(())
    }
    pub fn weapons_update(&mut self) -> anyhow::Result<()> {
        for (v, i) in &mut self.weapons.iter_mut().enumerate() {
            if !i.valid {
                get_server().weapons.remove(v);
                return Ok(());
            }
            i.move_loop()?;
        }
        Ok(())
    }
    pub fn maps_update(&mut self) -> anyhow::Result<()> {
        for (_, i) in &mut self.maps {
            i.update()?;
        }
        Ok(())
    }
    pub fn poll(&mut self) -> anyhow::Result<()> {
        self.data.bans.update()?;
        self.players_update()?;
        self.connections_update()?;
        self.weapons_update()?;
        self.maps_update()?;
        if let Some(recv) = self.host.service(1)? {
            match recv {
                Event::Receive {
                    ref sender,
                    ref packet,
                    ..
                } => {
                    if let Ok(data) = &get_server().foo.decrypto(packet.data()) {
                        self.process_packet(data, *sender)?;
                    }
                }
                Event::Connect(ref addr) => {
                    println!("New connection: {}", addr.address().ip());
                    self.add_connection(*addr);
                }
                Event::Disconnect(ref addr, _) => {
                    println!("New disconnection {}", addr.address().ip());
                    if self.get_connection(*addr).is_some() {
                        self.connections.remove(addr);
                    }
                    if let Some(player) = self.get_player(addr) {
                        player.save()?;
                        let name = player.name.clone();
                        get_server().players.remove(addr);
                        let mut offline = packets::Offline::default();
                        offline.who = name;
                        get_server().broadcast(Data::Offline(offline))?;
                    }
                }
            }
        }
        Ok(())
    }
}
