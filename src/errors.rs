use crate::message::Message;

#[derive(Debug)]
pub enum Error {
    PeerAlreadyConnected,
    PeerNotFound,
    CannotConnectPeer,
    CannotHandleMessage(Message),
    CannotRead(std::io::Error),
    CannotWrite(std::io::Error),
    CannotBind(std::io::Error),
    CannnotParseConfigFile(toml::de::Error),
    CannotGetLock,
    AuthenticationFailed,
    TransportError(snow::error::Error),
    UnknownMessage,
    UnsupportedOperation,
}
