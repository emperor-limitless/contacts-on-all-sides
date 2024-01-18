#![allow(unused)]
use crate::{
    dm::Dm,
    game::{packets, packets::packet::Data},
    network_state::NetworkState,
};
use enet::*;
use log::{error, warn};
use prost::Message;
use std::ffi::CString;
use std::{
    net::{SocketAddr, ToSocketAddrs},
    time::Instant,
};

pub struct Client {
    peer: Option<Peer>,
    host: Host<()>,
    pub state: NetworkState,
    pub packet: Option<Vec<u8>>,
    foo: Dm,
    timer: Instant,
}

impl Client {
    pub fn new(addr: &str, port: u16, enet: &mut Enet) -> anyhow::Result<Self> {
        let mut host = enet.create_host::<()>(
            None,
            10,
            ChannelLimit::Maximum,
            BandwidthLimit::Unlimited,
            BandwidthLimit::Unlimited,
        )?;
        host.connect(&Address::from_hostname(&CString::new(addr)?, port)?, 100, 0)?;
        Ok(Self {
            peer: None,
            host,
            state: NetworkState::Unconnected,
            packet: None,
            timer: Instant::now(),
            foo: Dm::new(
                String::new(),
                String::from("KcBkgRw_ju2TbjHc9V21VY9-bm0U2mRAKPZdM9aKZ_E="),
            ),
        })
    }
    pub fn send(&mut self, data: Data) -> anyhow::Result<()> {
        if let Some(peer) = self.peer.as_mut() {
            let mut pac = packets::Packet::default();
            pac.data = Some(data);
            let mut buf = vec![];
            pac.encode(&mut buf)?;
            let p = match Packet::new(
                self.foo.encrypto(buf.as_slice()).as_bytes(),
                PacketMode::ReliableSequenced,
            ) {
                Ok(p) => p,
                Err(e) => {
                    warn!("{}", e);
                    return Ok(());
                }
            };
            if let Err(e) = peer.send_packet(p, 0) {
                warn!("{}", e);
            }
        }
        Ok(())
    }
    pub fn poll(&mut self) -> anyhow::Result<()> {
        self.packet = None;
        let pac = match self.host.service(1) {
            Ok(pac) => pac,
            Err(e) => {
                warn!("{}", e);
                return Ok(());
            }
        };
        if let Some(recv) = pac {
            match recv {
                Event::Receive { ref packet, .. } => {
                    if let Ok(data) = self.foo.decrypto(packet.data()) {
                        self.packet = Some(data);
                    }
                }
                Event::Connect(ref peer) => {
                    self.state = NetworkState::RawConnection;
                    self.peer = Some(peer.clone());
                }
                _ => {
                    self.state = NetworkState::Unconnected;
                    println!("Disconnect");
                }
            }
        }
        Ok(())
    }
}
