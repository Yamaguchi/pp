use crate::crypto::curves::Ed25519;
use crate::key::PublicKey;
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct Peer {
    pub public_key: Option<PublicKey<Ed25519>>,
    pub addr: SocketAddr,
    pub state: PeerState,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Self {
        Peer {
            public_key: None,
            addr: addr,
            state: PeerState::Init,
        }
    }
}

#[derive(Clone, Debug)]
pub enum PeerState {
    Init,
}
