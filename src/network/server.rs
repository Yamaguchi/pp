use crate::configuration::ServerConfiguration;
use crate::grpc::network::initiate_response::Event;
use crate::grpc::network::{AlreadyConnected, Authenticated, Connected, Disconnected};
use crate::node::context::NODE;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio::sync::mpsc;

use crate::network::peer::Peer;

use crate::crypto::authenticator::Authenticator;

pub struct Server {
    pub configuration: ServerConfiguration,
}

impl Server {
    pub fn new(configuration: ServerConfiguration) -> Self {
        Server {
            configuration: configuration,
        }
    }
    async fn accept_loop(&mut self, addr: String) -> Result<(), std::io::Error> {
        let mut listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            // let (sender, receiver) = mpsc::channel(1);
            // tokio::spawn(async move {
            let _ = Server::accept(stream).await;
            // });
        }
        Ok(())
    }
    pub async fn start(&mut self) {
        info!("Start ...");
        let _ = self.accept_loop(self.configuration.address.clone()).await;
    }
    async fn accept(stream: TcpStream) -> Result<(), std::io::Error> {
        info!("{:?}", stream.peer_addr()?);

        let (sender, mut receiver) = mpsc::channel::<Event>(1);
        let addr: SocketAddr = stream.peer_addr()?;
        match NODE.write() {
            Ok(mut n) => {
                n.accept(addr, &stream, sender.clone()).await;
            }
            Err(_) => {}
        }
        while let Some(res) = receiver.recv().await {
            match res {
                Event::Connected(Connected { public_key }) => {
                    info!("Connected {}", public_key);
                    break;
                }
                Event::Disconnected(Disconnected { public_key }) => {
                    info!("Disconnected {}", public_key);
                    return Ok(());
                }
                Event::AlreadyConnected(AlreadyConnected { public_key }) => {
                    info!("AlreadyConnected {}", public_key);
                    return Ok(());
                }
                _ => {}
            }
        }

        // let cloned = peer.clone();
        let peer = Peer::new(addr);
        tokio::spawn(async move {
            let authenticator = Authenticator::new();
            let e = match authenticator.auth(&peer, &stream, false).await {
                Ok(_) => Event::Authenticated(Authenticated {
                    public_key: "".to_string(),
                    remote_public_key: "".to_string(),
                }),
                Err(_) => Event::Disconnected(Disconnected {
                    public_key: "".to_string(),
                }),
            };
            info!("Authenticated {:?}", e);
            let _ = Server::handle_client(stream).await;
        });
        Ok(())
    }
    async fn handle_client(_stream: TcpStream) -> Result<(), std::io::Error> {
        Ok(())
    }
}
