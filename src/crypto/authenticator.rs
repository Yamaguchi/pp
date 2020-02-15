use crate::errors::Error;
use tokio::net::TcpStream;

use crate::network::peer::Peer;

pub struct Authenticator;

impl Authenticator {
    pub fn new() -> Self {
        Authenticator {}
    }

    pub async fn auth(
        &self,
        peer: &Peer,
        _stream: &TcpStream,
        _initiator: bool,
    ) -> Result<(), Error> {
        debug!("auth: start {}", peer.addr);
        std::thread::sleep(std::time::Duration::from_secs(10));
        debug!("auth: end {}", peer.addr);
        Ok(())
    }
}
