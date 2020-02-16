use crate::errors::Error;
use crate::network::connection::Connection;
use crate::network::peer::Peer;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub struct Authenticator;

impl Authenticator {
    pub fn new() -> Self {
        Authenticator {}
    }

    pub async fn auth<T>(&self, peer: &Peer, connection: T, initiator: bool) -> Result<T, Error>
    where
        T: Connection,
    {
        let param = "Noise_XX_25519_ChaChaPoly_BLAKE2s"
            .parse::<snow::params::NoiseParams>()
            .map_err(|_| Error::AuthenticationFailed)?;
        let builder = snow::Builder::new(param).prologue("PingPongDash".as_bytes());
        let mut noise = if initiator {
            builder
                .build_initiator()
                .map_err(|_| Error::AuthenticationFailed)?
        } else {
            builder
                .build_responder()
                .map_err(|_| Error::AuthenticationFailed)?
        };
        let mut buf = [0u8; 65535];
        noise
            .write_message(&[], &mut buf)
            .map_err(|_| Error::AuthenticationFailed)?;
        connection.write(&mut buf);
        Ok(connection)
    }
}
