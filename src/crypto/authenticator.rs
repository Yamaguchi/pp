use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PrivateKey;
use crate::network::connection::Connection;
use bytes::BytesMut;
use snow::params::NoiseParams;
use snow::HandshakeState;
use snow::TransportState;

pub struct Authenticator {
    local_private_key: PrivateKey<Ed25519>,
}

impl Authenticator {
    pub fn new(local_private_key: PrivateKey<Ed25519>) -> Self {
        Authenticator {
            local_private_key: local_private_key,
        }
    }

    pub async fn auth<T>(
        &self,
        connection: &mut T,
        initiator: bool,
    ) -> Result<TransportState, Error>
    where
        T: Connection,
    {
        let mut handshake = self.start_handshake(connection, initiator)?;
        if initiator {
            handshake = self.act1(handshake, connection).await?;
        }
        let transport = self.act2(handshake, connection).await?;

        Ok(transport)
    }

    fn start_handshake<T>(&self, _connection: &T, initiator: bool) -> Result<HandshakeState, Error>
    where
        T: Connection,
    {
        //XX:
        //   -> e
        //   <- e, ee, s, es
        //   -> s, se
        let param = "Noise_XX_25519_ChaChaPoly_BLAKE2s"
            .parse::<NoiseParams>()
            .map_err(|_| Error::AuthenticationFailed)?;
        let key = self.local_private_key.clone();
        let builder = snow::Builder::new(param)
            .prologue("PingPongDash".as_bytes())
            .local_private_key(key.inner.as_slice());
        let handshake = if initiator {
            builder
                .build_initiator()
                .map_err(|_| Error::AuthenticationFailed)?
        } else {
            builder
                .build_responder()
                .map_err(|_| Error::AuthenticationFailed)?
        };
        Ok(handshake)
    }

    async fn act1<T>(
        &self,
        mut handshake: HandshakeState,
        connection: &mut T,
    ) -> Result<HandshakeState, Error>
    where
        T: Connection,
    {
        let mut buf = [0u8; 65535];
        handshake
            .write_message(&[], &mut buf)
            .map_err(|_| Error::AuthenticationFailed)?;
        let _ = connection.write(&mut buf).await;
        Ok(handshake)
    }

    async fn act2<T>(
        &self,
        mut handshake: HandshakeState,
        connection: &mut T,
    ) -> Result<TransportState, Error>
    where
        T: Connection,
    {
        while !handshake.is_handshake_finished() {
            let mut buf = [0u8; 65535];
            let mut incoming = BytesMut::with_capacity(65535);
            connection.read(incoming).await?;
            handshake
                .read_message(&incoming, &mut buf)
                .map_err(|_| Error::AuthenticationFailed)?;
            handshake
                .write_message(&[], &mut buf)
                .map_err(|_| Error::AuthenticationFailed)?;
        }

        Ok(handshake
            .into_transport_mode()
            .map_err(|_| Error::AuthenticationFailed)?)
    }
}
