use crate::application::Application;
use crate::configuration::ServerConfiguration;
use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PrivateKey;
use crate::network::connection::ConnectionImpl;
use crate::node::*;
use async_trait::async_trait;
use snow::TransportState;
use std::net::SocketAddr;
use std::ops::Deref;
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
        info!("listening... {}", addr);
        let mut listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();

        let key = {
            let guard_app = self.app.read().unwrap();
            let app = guard_app.deref();
            app.private_key()
        };
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            match self
                .accept(Arc::clone(&self.app), stream, key.clone())
                .await
            {
                Ok(_) => {}
                Err(e) => error!("{:?}", e),
            }
        }
        Ok(())
    }
    pub async fn start(&mut self) {
        info!("Start ...");
        let result = self.accept_loop(self.configuration.address.clone()).await;
        info!("End ... {:?}", result);
    }
    async fn accept(
        &self,
        app: Arc<RwLock<A>>,
        stream: TcpStream,
        key: PrivateKey<Ed25519>,
    ) -> Result<(), Error>
    where
        A: Application + 'static + Send + Sync,
    {
        let addr: SocketAddr = stream.peer_addr().map_err(|_| Error::CannotConnectPeer)?;
        info!("accept ... {:?}", addr);
        let peer = add_peer(Arc::clone(&app), addr)?;
        let mut server = ConnectionImpl::new(stream);
        let authenticator = Authenticator::new(key);
        let transport = authenticator.auth(&mut server, false).await?;
        server.transport = Some(transport);
        add_connection(Arc::clone(&app), peer.addr, server)
    }
    async fn handle_client(&self, connection: &ConnectionImpl) -> Result<(), std::io::Error> {
        Ok(())
    }
}
