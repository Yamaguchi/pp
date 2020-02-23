use crate::crypto::curves::Ed25519;
use crate::key::PublicKey;
use crate::message::Message;
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;

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

    pub async fn handle_request(m: Message, mut tx: Sender<Message>) {
        match m {
            Message::RequestPing => {
                let ping = Message::build_ping();
                tx.send(ping).await.ok();
            }
            Message::Ping(nonce) => {
                let pong = Message::Pong(nonce);
                tx.send(pong).await.ok();
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug)]
pub enum PeerState {
    Init,
}
