use crate::crypto::curves::Ed25519;
use crate::key::PublicKey;
use crate::message::Message;
use std::net::SocketAddr;
use tokio::sync::mpsc::UnboundedSender;

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

    pub async fn handle_message(m: Message, tx: UnboundedSender<Message>) {
        info!("handle_message: {:?}", m);
        match m {
            Message::RequestPing => {
                let ping = Message::build_ping();
                tx.send(ping).ok();
            }
            Message::Ping(nonce) => {
                let pong = Message::Pong(nonce);
                tx.send(pong).ok();
            }
            Message::Pong(_) => {
                // Do nothing.
            }
            Message::Data(data) => {
                info!("receive data: {:?}", hex::encode(data));
            }
            Message::RequestData(data) => {
                let data = Message::Data(data);
                tx.send(data).ok();
            }
            _ => panic!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum PeerState {
    Init,
}
