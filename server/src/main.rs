mod bans;
mod connection;
mod dm;
mod inventory;
mod items;
mod maps;
mod player;
mod readable_time;
mod rotation;
mod server;
mod timer;
mod weapon;
fn main() -> anyhow::Result<()> {
    let sr = server::get_server();
    loop {
        sr.poll()?;
    }
}
