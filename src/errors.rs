#[derive(Debug)]
pub enum Error {
    PeerAlreadyConnected,
    CannotConnectPeer,
    CannotRead,
    CannotGetLock,
    AuthenticationFailed,
    PeerNotFound,
}
