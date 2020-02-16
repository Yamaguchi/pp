use crate::errors::Error;
use crate::network::connection::Connection;
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub struct Client {
    pub stream: Option<TcpStream>,
}

impl Client {
    pub fn new() -> Self {
        Client { stream: None }
    }

    pub async fn connect(&mut self, addr: SocketAddr) -> Result<(), Error> {
        if let Ok(stream) = TcpStream::connect(addr).await {
            self.stream = Some(stream);
            Ok(())
        } else {
            Err(Error::CannotConnectPeer)
        }
    }
}

impl Connection for Client {
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
