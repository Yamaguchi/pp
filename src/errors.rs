use crate::network::client::Client;
use crate::network::server::ServerConnection;

pub enum Error {
    PeerAlreadyConnected,
    CannotConnectPeer,
    PeerNotFound,
    CannotGetLock,
    ServerAuthenticationFailed(ServerConnection),
    ClientAuthenticationFailed(Client),
    AuthenticationFailed,
}
