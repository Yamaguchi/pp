use crate::application::Application;
use crate::errors::Error;
use crate::network::connection::ConnectionImpl;
use crate::network::peer::Peer;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};

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
