#[derive(Eq, PartialEq, Hash)]
pub enum NetworkState {
    Unconnected,
    RawConnection,
    Connected,
    AwaitingResponse,
}
