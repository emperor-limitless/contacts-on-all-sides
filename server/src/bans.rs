use crate::{server::get_server, timer::Timer};
use serde_derive::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Default)]
pub struct BannedUser {
    name: String,
    id: String,
}
#[derive(Serialize, Deserialize, Default)]
pub struct TemporaryBan {
    name: String,
    time: u64,
    timer: Timer,
    id: String,
}
#[derive(Serialize, Deserialize, Default)]
pub struct Bans {
    users: Vec<BannedUser>,
    #[serde(default)]
    temporary: Vec<TemporaryBan>,
}

impl Bans {
    pub fn get_user(&self, id: String) -> bool {
        for i in &self.users {
            if id == i.id {
                return true;
            }
        }
        false
    }
    pub fn add_ban(&mut self, name: String, id: String) {
        let user = BannedUser { name, id };
        self.users.push(user);
    }
    pub fn remove_ban(&mut self, name: String) -> bool {
        for i in 0..self.users.len() {
            if self.users[i].name == name {
                self.users.remove(i);
                return true;
            }
        }
        false
    }
    pub fn add_temporary(&mut self, name: String, id: String, time: u64) {
        let temp = TemporaryBan {
            name,
            id,
            time,
            timer: Timer::new(),
        };
        self.temporary.push(temp);
    }
    pub fn remove_temporary(&mut self, name: String) -> bool {
        for i in 0..self.temporary.len() {
            if self.temporary[i].name == name {
                self.temporary.remove(i);
                return true;
            }
        }
        false
    }
    pub fn get_temporary(&self, id: String) -> bool {
        for i in &self.temporary {
            if id == i.id {
                return true;
            }
        }
        false
    }
    pub fn get_temporary_remaining(&self, id: String) -> u64 {
        for i in &self.temporary {
            if id == i.id {
                return i.time - i.timer.elapsed();
            }
        }
        0
    }
    pub fn update(&mut self) -> anyhow::Result<()> {
        for i in 0..self.temporary.len() {
            if self.temporary[i].timer.elapsed() >= self.temporary[i].time {
                get_server().notify(format!(
                    "{}'s temporary ban have expired",
                    self.temporary[i].name
                ))?;
                self.temporary.remove(i);
                return Ok(());
            }
        }
        Ok(())
    }
}
