use crate::application::Application;
use crate::errors::Error;
use crate::message::Message;
use crate::network::connection::Connection;
use crate::network::connection::ConnectionImpl;
use crate::network::peer::Peer;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc::channel;
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
        mut connection: ConnectionImpl,
    ) -> Result<(), Error> {
        if self.connections.contains_key(&addr) {
            return Err(Error::PeerAlreadyConnected);
        }
        // self.connections.insert(addr, connection);

        //channel for send Message to other node.
        let (send_tx, mut send_rx) = channel::<Message>(1);

        //channel for recv Message from other node.
        let (recv_tx, mut recv_rx) = channel::<Message>(1);
        let mut tx = recv_tx.clone();
        tokio::spawn(async move {
            let mut buffer = [0u8; 65535];
            while let Some(m) = send_rx.recv().await {
                connection.send_message(m).await.unwrap();
            }
            while let Ok(r) = connection.receive_message(&mut buffer).await {
                match r {
                    (Some(m), rest) => {
                        buffer = [0u8; 65535];
                        buffer.copy_from_slice(&rest[..]);
                        tx.send(m.clone()).await.ok();
                    }
                    (None, rest) => {
                        buffer = [0u8; 65535];
                        buffer.copy_from_slice(&rest[..]);
                    }
                }
            }
        });
        let mut tx = recv_tx.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                tx.send(Message::RequestPing).await.ok();
            }
        });

        tokio::spawn(async move {
            while let Some(m) = recv_rx.recv().await {
                Peer::handle_request(m, send_tx.clone()).await;
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
    let _ = node.add_connection(addr, conn);
    Ok(())
}
