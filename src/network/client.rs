use crate::errors::Error;
use crate::network::connection::Connection;
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub struct Client {
    pub stream: TcpStream,
}

impl Client {
    pub async fn connect(addr: SocketAddr) -> Result<Self, Error> {
        if let Ok(stream) = TcpStream::connect(addr).await {
            Ok(Client { stream: stream })
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
