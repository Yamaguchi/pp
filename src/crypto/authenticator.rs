use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PrivateKey;
use crate::network::connection::*;
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

    pub async fn auth(
        &self,
        connection: &mut ConnectionImpl,
        initiator: bool,
    ) -> Result<TransportState, Error> {
        info!("auth ... ");
        let mut handshake = self.start_handshake(initiator)?;
        if initiator {
            handshake = self.act1(handshake, connection).await?;
        }
        let transport = self.act2(handshake, connection).await?;

        Ok(transport)
    }

    fn start_handshake(&self, initiator: bool) -> Result<HandshakeState, Error> {
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

    async fn act1(
        &self,
        mut handshake: HandshakeState,
        connection: &mut ConnectionImpl,
    ) -> Result<HandshakeState, Error> {
        let mut buf = [0u8; 65535];
        let len = handshake
            .write_message(&[], &mut buf)
            .map_err(|_| Error::AuthenticationFailed)?;
        connection.write(&mut buf[0..len]).await?;
        Ok(handshake)
    }

    async fn act2(
        &self,
        mut handshake: HandshakeState,
        connection: &mut ConnectionImpl,
    ) -> Result<TransportState, Error> {
        while !handshake.is_handshake_finished() {
            let mut buf = [0u8; 65535];
            let incoming = connection.read().await?;
            let _ = handshake
                .read_message(&incoming[..], &mut buf)
                .map_err(|_| Error::AuthenticationFailed)?;
            if handshake.is_handshake_finished() {
                break;
            }
            let len = handshake
                .write_message(&[], &mut buf)
                .map_err(|_| Error::AuthenticationFailed)?;
            let _ = connection.write(&mut buf[0..len]).await;
        }
        Ok(handshake
            .into_transport_mode()
            .map_err(|_| Error::AuthenticationFailed)?)
    }
}
