use enet_sys::{
    ENetEvent, _ENetEventType_ENET_EVENT_TYPE_CONNECT, _ENetEventType_ENET_EVENT_TYPE_DISCONNECT,
    _ENetEventType_ENET_EVENT_TYPE_NONE, _ENetEventType_ENET_EVENT_TYPE_RECEIVE,
};

use crate::{Packet, Peer};

/// This enum represents an event that can occur when servicing an `EnetHost`.
///
/// Also see the official ENet documentation for more information.
#[derive(Debug)]
pub enum Event {
    /// This variant represents the connection of a peer, contained in the only
    /// field.
    Connect(Peer),
    /// This variant represents the disconnection of a peer, either because it
    /// was requested or due to a timeout.
    ///
    /// The disconnected peer is contained in the first field, while the second
    /// field contains the user-specified data for this disconnection.
    Disconnect(Peer, u32),
    /// This variants repersents a packet that was received.
    Receive {
        /// The `Peer` that sent the packet.
        sender: Peer,
        /// The channel on which the packet was received.
        channel_id: u8,
        /// The `Packet` that was received.
        packet: Packet,
    },
}

impl Event {
    pub(crate) fn from_sys_event<'b>(event_sys: &'b ENetEvent) -> Option<Event> {
        #[allow(non_upper_case_globals)]
        match event_sys.type_ {
            _ENetEventType_ENET_EVENT_TYPE_NONE => None,
            _ENetEventType_ENET_EVENT_TYPE_CONNECT => {
                Some(Event::Connect(Peer::new(event_sys.peer)))
            }
            _ENetEventType_ENET_EVENT_TYPE_DISCONNECT => {
                Some(Event::Disconnect(Peer::new(event_sys.peer), event_sys.data))
            }
            _ENetEventType_ENET_EVENT_TYPE_RECEIVE => Some(Event::Receive {
                sender: Peer::new(event_sys.peer),
                channel_id: event_sys.channelID,
                packet: Packet::from_sys_packet(event_sys.packet),
            }),
            _ => panic!("unrecognized event type: {}", event_sys.type_),
        }
    }
}

impl Drop for Event {
    fn drop(&mut self) {
    }
}
