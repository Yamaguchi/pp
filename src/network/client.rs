use crate::crypto::authenticator::Authenticator;
use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PrivateKey;
use crate::network::connection::ConnectionImpl;
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub struct Client {}

impl Client {
    pub async fn connect(
        addr: SocketAddr,
        key: PrivateKey<Ed25519>,
    ) -> Result<ConnectionImpl, Error> {
        info!("connecting...");
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|_| Error::CannotConnectPeer)?;
        info!("connected...");
        let authenticator = Authenticator::new(key);
        let mut client = ConnectionImpl::new(stream);
        let transport = authenticator.auth(&mut client, true).await?;
        client.transport = Some(transport);
        Ok(client)
    }
}
