use crate::errors::Error;
use async_trait::async_trait;
use snow::TransportState;
use std::marker::Sized;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;

#[async_trait]
pub trait Connection: Sized {
    async fn write(&mut self, buf: &[u8]) -> Result<(), Error>;
    async fn read(&mut self) -> Result<Vec<u8>, Error>;
    async fn shutdown(&mut self) -> Result<(), Error>;
}

pub struct ConnectionImpl {
    pub stream: TcpStream,
    pub transport: Option<TransportState>,
}
impl ConnectionImpl {
    pub fn new(stream: TcpStream) -> Self {
        ConnectionImpl {
            stream: stream,
            transport: None,
        }
    }
}
#[async_trait]
impl Connection for ConnectionImpl {
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
