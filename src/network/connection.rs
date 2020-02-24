use crate::crypto::transporter::Transporter;
use crate::errors::Error;
use crate::message::Message;
use async_trait::async_trait;
use snow::TransportState;
use std::io;
use std::ops::DerefMut;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::stream::Stream;
use tokio::sync::mpsc::UnboundedReceiver;

#[async_trait]
pub trait Connection {
    async fn write(&mut self, buf: &[u8]) -> Result<(), Error>;
    async fn read(&mut self) -> Result<Vec<u8>, Error>;
    async fn shutdown(&mut self) -> Result<(), Error>;
    async fn send_message(&mut self, message: Message) -> Result<(), Error>;
    async fn receive_message(
        &mut self,
        rest: &mut Vec<u8>,
    ) -> Result<(Option<Message>, Vec<u8>), Error>;
}

pub struct ConnectionImpl {
    pub stream: TcpStream,
    pub transport: Option<TransportState>,
    pub sender: Option<Arc<Mutex<UnboundedReceiver<Message>>>>,
}

impl ConnectionImpl {
    pub fn new(stream: TcpStream) -> Self {
        ConnectionImpl {
            stream: stream,
            transport: None,
            sender: None,
        }
    }
}

#[async_trait]
impl Connection for ConnectionImpl {
    async fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        trace!(
            "[{:?}]: write {:?}",
            self.stream.peer_addr(),
            hex::encode(buf)
        );
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
        trace!(
            "[{:?}]: read {:?}",
            self.stream.peer_addr(),
            hex::encode(&buf[0..n])
        );
        Ok(Vec::from(&buf[..n]))
    }
    async fn send_message(&mut self, message: Message) -> Result<(), Error> {
        let mut t = self.transport.as_mut().unwrap();
        let buf = Transporter::write_message(&mut t, message.clone())?;
        self.stream.write(&buf[..]).await.unwrap();
        trace!(
            "[{:?}]: send_message: {:?}, {:?}",
            self.stream.peer_addr(),
            message,
            hex::encode(&buf[..])
        );
        Ok(())
    }

    async fn receive_message(
        &mut self,
        rest: &mut Vec<u8>,
    ) -> Result<(Option<Message>, Vec<u8>), Error> {
        let mut read_buffer = [0u8; 65535];
        let n = self
            .stream
            .read(&mut read_buffer)
            .await
            .map_err(|_| Error::CannotRead)?;
        let vec: Vec<u8> = rest.iter().chain(&read_buffer[0..n]).map(|&b| b).collect();
        let mut t = self.transport.as_mut().unwrap();
        let (message, rest) = Transporter::read_message(&mut t, &vec[0..rest.len() + n])?;
        trace!(
            "[{:?}]: receive_message: {:?}, {:?}",
            self.stream.peer_addr(),
            message,
            hex::encode(&rest[..])
        );
        Ok((message, rest))
    }
    async fn shutdown(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum Actions {
    Send(Message),
    Receive,
}
impl Stream for ConnectionImpl {
    type Item = Result<Actions, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // First poll the `UnboundedReceiver`.
        let arc = Arc::clone(&self.sender.as_ref().unwrap());
        let mut guard = arc.lock().unwrap();
        let mut sender = guard.deref_mut();
        if let Poll::Ready(Some(v)) = Pin::new(&mut sender).poll_next(cx) {
            return Poll::Ready(Some(Ok(Actions::Send(v))));
        }
        // Secondly poll the TcpStream.
        let mut buf = [0; 1];
        let result: Result<usize, io::Error> =
            futures::ready!(Pin::new(&mut self.stream).poll_peek(cx, &mut buf));

        match result {
            Ok(0) => Poll::Pending,
            Ok(_) => Poll::Ready(Some(Ok(Actions::Receive))),
            Err(_) => Poll::Ready(Some(Err(Error::CannotRead))),
        }
    }
}
