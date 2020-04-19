use crate::application::Application;
use crate::configuration;
use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::event::Event;
use crate::event::EventManager;
use crate::event::EventType;
use crate::key::PublicKey;
use crate::message::Message;
use crate::network::connection::Actions;
use crate::network::connection::Connection;
use crate::network::connection::ConnectionImpl;
use crate::network::peer::Peer;
use std::collections::HashMap;
use std::net::Shutdown;
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

pub struct Node {
    peers: HashMap<SocketAddr, Peer>,
    message_handlers: HashMap<PublicKey<Ed25519>, Sender<Message>>,
    config: configuration::Application,
}

impl Node {
    pub fn new(config: configuration::Application) -> Self {
        Node {
            peers: HashMap::<SocketAddr, Peer>::new(),
            message_handlers: HashMap::<PublicKey<Ed25519>, Sender<Message>>::new(),
            config: config,
        }
    }

    pub fn remove_connection(&mut self, addr: SocketAddr) -> Result<(), Error> {
        info!("Node#remove_connection");
        let peer = self.peers[&addr].clone();
        self.peers.remove(&addr);
        if let Some(key) = peer.public_key {
            self.message_handlers.remove(&key);
        }
        Ok(())
    }

    pub fn add_connection(&mut self, mut connection: ConnectionImpl) -> Result<(), Error> {
        info!("Node#add_connection");
        //channel for send Message to other node.
        let (send_tx, send_rx) = unbounded_channel::<Message>();

        //channel for recv Message from other node.
        let (recv_tx, mut recv_rx) = channel::<Message>(1);
        let mut tx = recv_tx.clone();
        connection.relayer = Some(Arc::new(Mutex::new(send_rx)));
        let key = connection.remote_static_key()?;
        info!("connection is established: {:?}", key.clone());
        let addr = connection
            .stream
            .peer_addr()
            .map_err(|_| Error::CannotConnectPeer)?;

        let cloned_send_tx = send_tx.clone();
        tokio::spawn(async move {
            let mut buffer = vec![];
            while let Some(result) = connection.next().await {
                match result {
                    Ok(action) => match action {
                        Actions::Send(Message::Disconnect) => {
                            EventManager::broadcast(Event::Disconnected(connection.addr)).unwrap();
                            match connection.stream.shutdown(Shutdown::Both) {
                                Ok(_) => {}
                                Err(e) => error!("failed to shutown {:?}", e),
                            }
                        }
                        Actions::Send(m) => match connection.send_message(m).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("failed to send message {:?}", e);
                                let _ = cloned_send_tx.clone().send(Message::Disconnect);
                            }
                        },
                        Actions::Receive => {
                            let recv = connection.receive_message(&mut buffer).await;
                            match recv {
                                Ok((Some(m), rest)) => {
                                    buffer = rest;
                                    match tx.send(m.clone()).await {
                                        Ok(_) => {}
                                        Err(e) => error!("failed to recv message {:?}", e),
                                    }
                                }
                                Ok((None, rest)) => {
                                    buffer = rest;
                                }
                                Err(e) => {
                                    error!("failed to recv message {:?}", e);
                                    let _ = cloned_send_tx.clone().send(Message::Disconnect);
                                }
                            }
                        }
                    },
                    Err(_) => {}
                }
            }
        });

        self.start_ping_thread(recv_tx.clone());

        let mut peer = self.peers[&addr].clone();
        peer.public_key = Some(key.clone());
        self.message_handlers.insert(key.clone(), recv_tx.clone());
        self.update_peer(addr, &peer)?;
        tokio::spawn(async move {
            while let Some(m) = recv_rx.recv().await {
                match peer.handle_message(m, send_tx.clone()).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Could not handle message: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    pub fn update_peer(&mut self, addr: SocketAddr, peer: &Peer) -> Result<(), Error> {
        if self.peers.contains_key(&addr) {
            self.peers.insert(addr, peer.clone());
        } else {
            return Err(Error::PeerNotFound);
        }
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

    fn start_ping_thread(&mut self, mut tx: Sender<Message>) {
        let interval = self.config.ping_interval;
        if interval > 0 {
            // let event_manager = Arc::clone(&event_manager);
            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(interval));
                let receiver = EventManager::subscribe(EventType::Disconnected).unwrap();

                loop {
                    interval.tick().await;
                    match receiver.try_recv() {
                        Ok(Event::Disconnected(_addr)) => {
                            info!("Node#start_ping_thread disconnected");
                            break;
                        }
                        _ => {}
                    }
                    info!("send Message::RequestPing");
                    match tx.send(Message::RequestPing).await {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("failed to send ping: {:?}", e);
                        }
                    }
                }
            });
        }
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
    let guard_app = app.read().map_err(|_| Error::CannotGetLock)?;
    let app = guard_app.deref();
    let mut guard_node = app.node().map_err(|_| Error::CannotGetLock)?;
    let node = guard_node.deref_mut();
    node.send_to_peer(message, key)
}

pub fn add_peer<A>(app: Arc<RwLock<A>>, addr: SocketAddr) -> Result<Peer, Error>
where
    A: Application + 'static + Send + Sync,
{
    let guard_app = app.read().map_err(|_| Error::CannotGetLock)?;
    let app = guard_app.deref();
    let mut guard_node = app.node().map_err(|_| Error::CannotGetLock)?;
    let node = guard_node.deref_mut();
    node.add_peer(addr)
}

pub fn add_connection<A>(app: Arc<RwLock<A>>, conn: ConnectionImpl) -> Result<(), Error>
where
    A: Application + 'static + Send + Sync,
{
    let guard_app = app.read().map_err(|_| Error::CannotGetLock)?;
    let app_ref = guard_app.deref();
    let mut guard_node = app_ref.node().map_err(|_| Error::CannotGetLock)?;
    let node = guard_node.deref_mut();
    let _ = node.add_connection(conn);
    Ok(())
}

pub fn remove_connection<A>(app: Arc<RwLock<A>>, addr: SocketAddr) -> Result<(), Error>
where
    A: Application + 'static + Send + Sync,
{
    let guard_app = app.read().map_err(|_| Error::CannotGetLock)?;
    let app = guard_app.deref();
    let mut guard_node = app.node().map_err(|_| Error::CannotGetLock)?;
    let node = guard_node.deref_mut();
    let _ = node.remove_connection(addr);
    Ok(())
}
