use crate::errors::Error;
use crate::network::connection::Connection;
use crate::node::Connections;
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub struct Client {
    pub stream: TcpStream,
}

impl Client {
    pub async fn connect(addr: SocketAddr) -> Result<Self, Error> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|_| Error::CannotConnectPeer)?;

        Ok(Client { stream: stream })
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
