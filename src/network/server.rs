use crate::application::Application;
use crate::configuration::ServerConfiguration;
use crate::errors::Error;
use crate::grpc::network::initiate_response::Event;
use crate::grpc::network::{AlreadyConnected, Authenticated, Connected, Disconnected};
use crate::network::connection::Connection;

use std::net::Shutdown;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
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
            let _ = self.accept(stream).await;
        }
        Ok(())
    }
    pub async fn start(&mut self) {
        info!("Start ...");
        let _ = self.accept_loop(self.configuration.address.clone()).await;
    }
    async fn accept(&self, stream: TcpStream) -> Result<(), Error> {
        let (sender, mut receiver) = mpsc::channel::<Event>(1);
        let addr: SocketAddr = stream.peer_addr().map_err(|_| Error::CannotConnectPeer)?;
        let app = self.app.read().map_err(|x| Error::CannotGetLock)?;
        app.node()?.accept(addr, &stream, sender.clone()).await;
        while let Some(res) = receiver.recv().await {
            info!("{:?}", res);
            match res {
                Event::Connected(_) => {
                    break;
                }
                _ => return Ok(()),
            }
        }

        let peer = Peer::new(addr);

        let (mut sender, mut receiver) = mpsc::channel::<Result<ServerConnection, _>>(1);
        tokio::spawn(async move {
            let authenticator = Authenticator::new();
            let mut connection = ServerConnection { stream: stream };
            let e = authenticator.auth(&peer, connection, false).await;
            sender.send(e).await
        });
        match receiver.recv().await {
            Some(Ok(connection)) => {
                let app = self.app.read().map_err(|_| Error::CannotGetLock)?;
                app.node()?
                    .connections
                    .insert(addr.clone(), connection.stream);
                // let _ = self.handle_client(&connection).await;
            }
            Some(Err(Error::ServerAuthenticationFailed(connection))) => {
                connection.shutdown();
            }
            _ => {
                return Err(Error::AuthenticationFailed);
            }
        }
        Ok(())
    }
    async fn handle_client(&self, connection: &ServerConnection) -> Result<(), std::io::Error> {
        Ok(())
    }
}

pub struct ServerConnection {
    stream: TcpStream,
}

impl Connection for ServerConnection {
    fn write(&self, buf: &[u8]) -> Result<(), Error> {
        Ok(())
    }
    fn read(&self) -> Result<(), Error> {
        Ok(())
    }
    fn shutdown(&self) -> Result<(), Error> {
        Ok(())
    }
}
