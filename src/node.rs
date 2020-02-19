use crate::application::Application;
use crate::errors::Error;
use crate::message::Message;
use crate::network::client::Client;
use crate::network::peer::Peer;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};
use tokio::net::TcpStream;
pub enum Connections {
    Incomeing(TcpStream),
    Outgoing(Client),
}
// #[derive(Clone)]
pub struct Node {
    peers: HashMap<SocketAddr, Peer>,
    connections: HashMap<SocketAddr, Connections>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            peers: HashMap::<SocketAddr, Peer>::new(),
            connections: HashMap::<SocketAddr, Connections>::new(),
        }
    }

    pub fn add_connection(
        &mut self,
        addr: SocketAddr,
        connection: Connections,
    ) -> Result<(), Error> {
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

    pub fn send_to_peer<T: Message>(&self, addr: SocketAddr, message: T) -> Result<(), Error> {
        if let Some(peer) = self.peers.get(&addr) {
            peer.process(message);
            Ok(())
        } else {
            Err(Error::PeerNotFound)
        }
    }

    // pub fn schedule_ping(&self, peer: &Peer) -> Result<(), std::io::Error> {
    //     let mut rt = Runtime::new()?;
    //     let mut interval = time::interval(Duration::from_millis(10));
    //     rt.block_on(async {
    //         loop {
    //             interval.tick().await;

    //             let mut rt = Runtime::new().expect("failed to get runtime");
    //             rt.block_on(async {
    //                 let ping = Ping::new();
    //                 self.send_to_peer(&peer.key, ping);
    //             });
    //         }
    //     });
    //     Ok(())
    // }
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
    conn: Connections,
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
