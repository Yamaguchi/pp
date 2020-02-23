use crate::application::Application;
use crate::errors::Error;
use crate::message::Message;
use crate::message::Ping;
use crate::network::client::Client;
use crate::network::connection::ConnectionImpl;
use crate::network::peer::Peer;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time;

// #[derive(Clone)]
pub struct Node {
    peers: HashMap<SocketAddr, Peer>,
    connections: HashMap<SocketAddr, ConnectionImpl>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            peers: HashMap::<SocketAddr, Peer>::new(),
            connections: HashMap::<SocketAddr, ConnectionImpl>::new(),
        }
    }

    pub fn add_connection(
        &mut self,
        addr: SocketAddr,
        connection: ConnectionImpl,
    ) -> Result<(), Error> {
        if self.connections.contains_key(&addr) {
            return Err(Error::PeerAlreadyConnected);
        }
        self.connections.insert(addr, connection);
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

    pub fn update_peer(&mut self, peer: Peer) -> Result<(), Error> {
        let addr = peer.addr.clone();
        if !self.peers.contains_key(&addr) {
            return Err(Error::PeerNotFound);
        }
        self.peers.insert(addr, peer.clone());
        Ok(())
    }

    pub fn send_to_peer<T: Message>(&self, addr: &SocketAddr, message: T) -> Result<(), Error> {
        Ok(())
    }
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

pub fn add_connection<A>(
    app: Arc<RwLock<A>>,
    addr: SocketAddr,
    conn: ConnectionImpl,
) -> Result<(), Error>
where
    A: Application + 'static + Send + Sync,
{
    let guard_app = app.read().unwrap();
    let app = guard_app.deref();
    let mut guard_node = app.node().ok().unwrap();
    let node = guard_node.deref_mut();
    node.add_connection(addr, conn)
}

pub fn schedule_ping<A>(app: Arc<RwLock<A>>, addr: SocketAddr) -> Result<(), Error>
where
    A: Application + 'static + Send + Sync,
{
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(10));
        loop {
            interval.tick().await;
            let ping = Ping::new();
            let guard_app = app.read().unwrap();
            let app = guard_app.deref();
            let mut guard_node = app.node().ok().unwrap();
            let node = guard_node.deref_mut();
            node.send_to_peer(&addr, ping);
        }
    });
    Ok(())
}
