use crate::errors::Error;
use crate::message::Message;
use crate::network::peer::Peer;
use crate::network::peer::PeerState;

use crate::crypto::authenticator::Authenticator;
use crate::grpc::network::initiate_response::Event;
use crate::grpc::network::{AlreadyConnected, Authenticated, Connected, Disconnected};
use crate::network::client::Client;
use crate::network::connection::Connection;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

// #[derive(Clone)]
pub struct Node {
    peers: HashMap<SocketAddr, Peer>,
    pub connections: HashMap<SocketAddr, TcpStream>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            peers: HashMap::<SocketAddr, Peer>::new(),
            connections: HashMap::<SocketAddr, TcpStream>::new(),
        }
    }

    pub async fn accept(
        &mut self,
        addr: SocketAddr,
        stream: &TcpStream,
        mut sender: Sender<Event>,
    ) {
        let peer = Peer::new(addr);
        match self.add_peer(addr.clone(), peer.clone()) {
            Ok(_) => {
                let e = Event::Connected(Connected {
                    public_key: "".to_string(),
                });
                tokio::spawn(async move {
                    let _ = sender.send(e).await;
                });
            }
            Err(Error::PeerAlreadyConnected) => {
                let e = Event::AlreadyConnected(AlreadyConnected {
                    public_key: "".to_string(),
                });
                tokio::spawn(async move {
                    let _ = sender.send(e).await;
                });
                return;
            }
            _ => {}
        }
    }

    pub fn connect(&mut self, addr: SocketAddr, mut sender: Sender<Event>) {
        let peer = Peer::new(addr);
        match self.add_peer(addr.clone(), peer.clone()) {
            Err(Error::PeerAlreadyConnected) => {
                let e = Event::AlreadyConnected(AlreadyConnected {
                    public_key: "".to_string(),
                });
                tokio::spawn(async move {
                    let _ = sender.send(e).await;
                });
                return;
            }
            _ => {}
        }
        debug!("{:?}", self.peers);

        let (mut s, mut r) = mpsc::channel(1);
        let cloned = peer.clone();
        tokio::spawn(async move {
            let client = match Client::connect(addr.clone()).await {
                Ok(client) => {
                    let e = Event::Connected(Connected {
                        public_key: "".to_string(),
                    });
                    let _ = sender.send(e).await;
                    client
                }
                Err(_) => {
                    let e = Event::Disconnected(Disconnected {
                        public_key: "".to_string(),
                    });
                    let _ = sender.send(e).await;
                    return;
                }
            };
            let authenticator = Authenticator::new();
            let e = match authenticator.auth(&cloned, client, true).await {
                Ok(client) => {
                    let _ = s.send(client).await;
                    Event::Authenticated(Authenticated {
                        public_key: "".to_string(),
                        remote_public_key: "".to_string(),
                    })
                }
                Err(_) => Event::Disconnected(Disconnected {
                    public_key: "".to_string(),
                }),
            };
            let _ = sender.send(e).await;
        });

        if let Ok(client) = r.try_recv() {
            self.connections.insert(addr.clone(), client.stream);
        }
    }

    pub fn add_peer(&mut self, addr: SocketAddr, peer: Peer) -> Result<(), Error> {
        let addr = addr.clone();
        if self.peers.contains_key(&addr) {
            return Err(Error::PeerAlreadyConnected);
        }
        self.peers.insert(addr, peer.clone());
        Ok(())
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
