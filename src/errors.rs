#[derive(Debug)]
pub enum Error {
    PeerAlreadyConnected,
    CannotConnectPeer,
    CannotRead,
    CannotGetLock,
    AuthenticationFailed,
    TransportError(snow::error::Error),
    UnknownMessage,
}
