use enet::Peer;
use std::time::Instant;
pub struct Connection {
    pub addr: Peer,
    pub timer: Instant,
}
impl Connection {
    pub fn new(addr: Peer) -> Self {
        Self {
            addr,
            timer: Instant::now(),
        }
    }
    pub fn update(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
