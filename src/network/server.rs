use crate::application::Application;
use crate::configuration::ServerConfiguration;
use crate::errors::Error;
use crate::grpc::network::initiate_response::Event;
use crate::grpc::network::{AlreadyConnected, Authenticated, Connected, Disconnected};
use crate::network::connection::Connection;
use crate::node::Connections;
use crate::node::*;
use async_trait::async_trait;
use bytes::BytesMut;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::stream;
use tokio::stream::StreamExt;

use tokio::sync::mpsc;

use crate::network::peer::Peer;

use crate::crypto::authenticator::Authenticator;

pub struct Server<A>
where
    A: Application + 'static + Send + Sync,
{
    app: Arc<RwLock<A>>,
    pub configuration: ServerConfiguration,
}

impl<A> Server<A>
where
    A: Application + 'static + Send + Sync,
{
    pub fn new(app: Arc<RwLock<A>>, configuration: ServerConfiguration) -> Self {
        Server::<A> {
            app: app,
            configuration: configuration,
        }
    }
    async fn accept_loop(&mut self, addr: String) -> Result<(), std::io::Error> {
        let mut listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            let _ = self.accept(Arc::clone(&self.app), stream).await;
        }
        Ok(())
    }
    pub async fn start(&mut self) {
        info!("Start ...");
        let _ = self.accept_loop(self.configuration.address.clone()).await;
    }
    async fn accept(&self, app: Arc<RwLock<A>>, stream: TcpStream) -> Result<(), Error>
    where
        A: Application + 'static + Send + Sync,
    {
        let addr: SocketAddr = stream.peer_addr().map_err(|_| Error::CannotConnectPeer)?;
        let peer = add_peer(Arc::clone(&app), addr)?;
        let server = Connections::Incomeing(stream);
        add_connection(Arc::clone(&app), peer.addr, server)
    }
    async fn handle_client(&self, connection: &ServerConnection) -> Result<(), std::io::Error> {
        Ok(())
    }
}

pub struct ServerConnection {
    stream: TcpStream,
}

#[async_trait]
impl Connection for ServerConnection {
    async fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        let _ = self.stream.write(buf).await;
        Ok(())
    }
    async fn read(&mut self, mut buf: BytesMut) -> Result<usize, Error> {
        self.stream.read_buf(&mut buf).await;
        Ok(1)
    }
    async fn shutdown(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
