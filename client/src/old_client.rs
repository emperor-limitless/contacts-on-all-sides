#![allow(unused)]
use crate::network_state::NetworkState;
use laminar::{Packet, Socket, SocketEvent};
use log::error;
use std::{
    net::{SocketAddr, ToSocketAddrs},
    time::Instant,
};

pub struct Client {
    socket: Socket,
    address: SocketAddr,
    pub state: NetworkState,
    pub packet: Option<String>,
}

impl Client {
    pub fn new(host: &str, port: i32) -> anyhow::Result<Self> {
        let server_details = &format!("{}:{}", host, port);
        let server: Vec<_> = server_details
            .to_socket_addrs()
            .expect("Unable to resolve domain")
            .collect();
        let addr = match server.into_iter().next() {
            Some(server) => server,
            None => {
                error!("Unable to find a server dns address");
                return Err(anyhow::format_err!("Server dns address unavailable..."));
            }
        };
        let socket = Socket::bind_any()?;
        Ok(Self {
            socket,
            address: addr,
            state: NetworkState::Unconnected,
            packet: None,
        })
    }
    pub fn send(&mut self, data: &str) -> anyhow::Result<()> {
        let packet = Packet::reliable_ordered(self.address, data.to_string().into_bytes(), Some(0));
        self.socket.send(packet)?;
        Ok(())
    }
    pub fn poll(&mut self) -> anyhow::Result<()> {
        self.packet = None;
        self.socket.manual_poll(Instant::now());
        if let Some(recv) = self.socket.recv() {
            match recv {
                SocketEvent::Packet(packet) => {
                    self.packet = Some(String::from_utf8_lossy(packet.payload()).to_string());
                }
                SocketEvent::Connect(addr) => {
                    self.state = NetworkState::RawConnection;
                    println!("Ziftaziftazifta");
                }
                _ => self.state = NetworkState::Unconnected,
            }
        }
        Ok(())
    }
}
