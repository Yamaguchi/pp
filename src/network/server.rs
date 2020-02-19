use crate::application::Application;
use crate::configuration::ServerConfiguration;
use crate::errors::Error;
use crate::network::connection::Connection;
use crate::node::Connections;
use crate::node::*;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
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
    async fn read(&mut self) -> Result<Vec<u8>, Error> {
        let mut buf = [0u8; 65535];
        let n = self
            .stream
            .read(&mut buf)
            .await
            .map_err(|_| Error::CannotRead)?;
        Ok(Vec::from(&buf[..n]))
    }
    async fn shutdown(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
