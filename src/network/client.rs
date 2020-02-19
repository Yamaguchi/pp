use crate::crypto::authenticator::Authenticator;
use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PrivateKey;
use crate::network::connection::Connection;
use crate::node::Connections;
use async_trait::async_trait;
use snow::TransportState;
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub struct Client {
    pub stream: TcpStream,
    pub transport: Option<TransportState>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client {
            stream: stream,
            transport: None,
        }
    }
    pub async fn connect(addr: SocketAddr, key: PrivateKey<Ed25519>) -> Result<Self, Error> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|_| Error::CannotConnectPeer)?;
        let authenticator = Authenticator::new(key);
        let mut client = Client::new(stream);
        let transport = authenticator.auth(&client, true).await?;
        client.transport = Some(transport);
        Ok(client)
    }
}

#[async_trait]
impl Connection for Client {
    async fn write(&self, buf: &[u8]) -> Result<(), Error> {
        Ok(())
    }
    async fn read(&self, mut buf: &[u8]) -> Result<usize, Error> {
        Ok(1)
    }
    async fn shutdown(&self) -> Result<(), Error> {
        Ok(())
    }
}
