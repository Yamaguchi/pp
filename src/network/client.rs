use crate::crypto::authenticator::Authenticator;
use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PrivateKey;
use crate::network::connection::Connection;
use async_trait::async_trait;
use snow::TransportState;
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
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
        let transport = authenticator.auth(&mut client, true).await?;
        client.transport = Some(transport);
        Ok(client)
    }
}

#[async_trait]
impl Connection for Client {
    async fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        trace!("write {:?}", hex::encode(buf));
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
        trace!("read {:?}", hex::encode(&buf[0..n]));
        Ok(Vec::from(&buf[..n]))
    }
    async fn shutdown(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
