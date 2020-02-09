use crate::configuration::ServerConfiguration;
use crate::message::Pong;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;

pub struct Server {
    pub configuration: ServerConfiguration,
}

impl Server {
    async fn accept_loop(&self, addr: String) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            async_std::task::spawn(async move {
                let _ = handle_client(stream).await;
            });
        }
        Ok(())
    }
    pub fn start(&self) {
        println!("Start ...");
        async_std::task::block_on(async {
            let _ = self.accept_loop(self.configuration.address.clone()).await;
        });
    }
}
async fn handle_client(mut stream: TcpStream) -> Result<(), std::io::Error> {
    let mut buffer = [0; 4];
    let mut i = 0u32;
    loop {
        stream.read(&mut buffer).await?;
        let mut array = [0; 4];
        array.copy_from_slice(&buffer);
        let nonce = u32::from_be_bytes(array);
        println!("Pong {} {} {:?}", stream.peer_addr()?, i, nonce);
        let pong = Pong { nonce: nonce };
        stream.write(&pong.nonce.to_be_bytes()).await?;
        stream.flush().await?;
        i += 1;
    }
}
