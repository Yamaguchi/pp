use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PublicKey;
use crate::message::Message;
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug)]
pub struct Peer {
    pub public_key: Option<PublicKey<Ed25519>>,
    pub addr: SocketAddr,
    pub state: PeerState,
    broadcaster: Option<Sender<Message>>,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Self {
        Peer {
            public_key: None,
            addr: addr,
            state: PeerState::Init,
            broadcaster: None,
        }
    }

    pub async fn handle_message(
        &mut self,
        m: Message,
        tx: UnboundedSender<Message>,
    ) -> Result<(), Error> {
        info!("handle_message: {:?}", m);
        match m.clone() {
            Message::RequestPing => {
                let ping = Message::build_ping();
                tx.send(ping)
                    .map_err(|_| Error::CannotHandleMessage(m.clone()))?;
            }
            Message::Ping(nonce) => {
                let pong = Message::Pong(nonce);
                tx.send(pong)
                    .map_err(|_| Error::CannotHandleMessage(m.clone()))?;
            }
            Message::Pong(_) => {
                // Do nothing.
            }
            Message::Data(data) => {
                info!("receive data: {:?}", hex::encode(data));
            }
            Message::RequestData(data) => {
                let data = Message::Data(data);
                tx.send(data)
                    .map_err(|_| Error::CannotHandleMessage(m.clone()))?;
            }
            Message::RequestSubscribe(sender) => self.broadcaster = Some(sender),
        }
        let broadcaster = self.broadcaster.clone();
        if let Some(mut broadcaster) = broadcaster {
            if m.is_p2p() {
                broadcaster
                    .send(m.clone())
                    .await
                    .map_err(|_| Error::CannotHandleMessage(m.clone()))?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum PeerState {
    Init,
}
