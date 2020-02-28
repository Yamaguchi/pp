use crate::application::Application;
use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PublicKey;
use crate::message::Message;
use crate::network::connection::Actions;
use crate::network::connection::Connection;
use crate::network::connection::ConnectionImpl;
use crate::network::peer::Peer;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::stream::StreamExt;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::Sender;
use tokio::time;

// #[derive(Clone)]
pub struct Node {
    peers: HashMap<SocketAddr, Peer>,
    message_handlers: HashMap<PublicKey<Ed25519>, Sender<Message>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            peers: HashMap::<SocketAddr, Peer>::new(),
            message_handlers: HashMap::<PublicKey<Ed25519>, Sender<Message>>::new(),
        }
    }

    pub fn add_connection(&mut self, mut connection: ConnectionImpl) -> Result<(), Error> {
        //channel for send Message to other node.
        let (send_tx, send_rx) = unbounded_channel::<Message>();

        //channel for recv Message from other node.
        let (recv_tx, mut recv_rx) = channel::<Message>(1);
        let mut tx = recv_tx.clone();
        connection.relayer = Some(Arc::new(Mutex::new(send_rx)));
        let key = connection.remote_static_key();
        info!("remote static key is {:?}", key);
        self.message_handlers.insert(key.unwrap(), recv_tx.clone());

        tokio::spawn(async move {
            // loop {
            let mut buffer = vec![];
            while let Some(result) = connection.next().await {
                match result {
                    Ok(action) => match action {
                        Actions::Send(m) => {
                            connection.send_message(m).await.unwrap();
                        }
                        Actions::Receive => {
                            let recv = connection.receive_message(&mut buffer).await;
                            match recv {
                                Ok((Some(m), rest)) => {
                                    buffer = rest;
                                    tx.send(m.clone()).await.ok();
                                }
                                Ok((None, rest)) => {
                                    buffer = rest;
                                }
                                Err(e) => {
                                    error!("{:?}", e);
                                }
                            }
                        }
                    },
                    Err(_) => {}
                }
            }
        });
        let mut tx = recv_tx.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                info!("send Message::RequestPing");
                tx.send(Message::RequestPing).await.ok();
            }
        });

        tokio::spawn(async move {
            while let Some(m) = recv_rx.recv().await {
                Peer::handle_message(m, send_tx.clone()).await;
            }
        });

        Ok(())
    }

    pub fn add_peer(&mut self, addr: SocketAddr) -> Result<Peer, Error> {
        let peer = Peer::new(addr);
        if self.peers.contains_key(&addr) {
            return Err(Error::PeerAlreadyConnected);
        }
        self.peers.insert(addr, peer.clone());
        Ok(peer)
    }
    fn send_to_peer(&mut self, message: Message, key: &PublicKey<Ed25519>) -> Result<(), Error> {
        let cloned_key = key.clone();
        let mut handler = self.message_handlers[key].clone();
        tokio::spawn(async move {
            match handler.send(message).await {
                Ok(()) => {}
                Err(e) => {
                    error!("[{:?}] send_to_peer: {:?}", cloned_key.clone(), e);
                }
            }
        });
        Ok(())
    }
}

pub fn send_to_peer<A>(
    app: Arc<RwLock<A>>,
    message: Message,
    key: &PublicKey<Ed25519>,
) -> Result<(), Error>
where
    A: Application + 'static + Send + Sync,
{
    let guard_app = app.read().unwrap();
    let app = guard_app.deref();
    let mut guard_node = app.node().ok().unwrap();
    let node = guard_node.deref_mut();
    node.send_to_peer(message, key)
}

pub fn add_peer<A>(app: Arc<RwLock<A>>, addr: SocketAddr) -> Result<Peer, Error>
where
    A: Application + 'static + Send + Sync,
{
    let guard_app = app.read().unwrap();
    let app = guard_app.deref();
    let mut guard_node = app.node().ok().unwrap();
    let node = guard_node.deref_mut();
    node.add_peer(addr)
}

pub fn add_connection<A>(app: Arc<RwLock<A>>, conn: ConnectionImpl) -> Result<(), Error>
where
    A: Application + 'static + Send + Sync,
{
    let guard_app = app.read().unwrap();
    let app = guard_app.deref();
    let mut guard_node = app.node().ok().unwrap();
    let node = guard_node.deref_mut();
    let _ = node.add_connection(conn);
    Ok(())
}
