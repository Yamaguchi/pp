use crate::crypto::transporter::Transporter;
use crate::errors::Error;
use crate::message::Message;
use async_trait::async_trait;
use snow::TransportState;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[async_trait]
pub trait Connection {
    async fn write(&mut self, buf: &[u8]) -> Result<(), Error>;
    async fn read(&mut self) -> Result<Vec<u8>, Error>;
    async fn shutdown(&mut self) -> Result<(), Error>;
    async fn send_message(&mut self, message: Message) -> Result<(), Error>;
    async fn receive_message(
        &mut self,
        rest: &mut [u8],
    ) -> Result<(Option<Message>, Vec<u8>), Error>;
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
        self.stream.write(buf).await.unwrap();
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
    async fn send_message(&mut self, message: Message) -> Result<(), Error> {
        let mut t = self.transport.as_mut().unwrap();
        let buf = Transporter::write_message(&mut t, message)?;
        self.stream.write(&buf[..]).await.unwrap();
        Ok(())
    }
    async fn receive_message(
        &mut self,
        rest: &mut [u8],
    ) -> Result<(Option<Message>, Vec<u8>), Error> {
        let mut read_buffer = [0u8; 65535];
        let n = self
            .stream
            .read(&mut read_buffer)
            .await
            .map_err(|_| Error::CannotRead)?;
        let mut buffer = [0u8; 65535 * 2];
        let vec: Vec<u8> = rest.iter().chain(&read_buffer[0..n]).map(|&b| b).collect();
        buffer.copy_from_slice(&vec[..]);
        let mut t = self.transport.as_mut().unwrap();
        let (message, rest) = Transporter::read_message(&mut t, &buffer[0..rest.len() + n])?;
        Ok((message, rest))
    }
    async fn shutdown(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
