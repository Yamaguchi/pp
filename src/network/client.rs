use rand::Rng;
use std::thread;
use std::time::Duration;

use async_std::io::BufWriter;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;

use crate::configuration::*;
use crate::message::*;

pub struct Client {
    pub configuration: ClientConfiguration,
}

impl Client {
    pub fn start(&self) {
        let address = self.configuration.address.clone();
        async_std::task::spawn(async {
            task::sleep(Duration::from_secs(1)).await;
            let stream = connect(address).await;
            if let Ok(stream) = stream {
                let mut i = 0u32;
                loop {
                    task::sleep(Duration::from_secs(1)).await;
                    async_std::task::block_on(async {
                        let _ = ping(&stream, i).await;
                    });
                    i += 1;
                }
            }
        });
    }
}

async fn connect(address: String) -> Result<TcpStream, std::io::Error> {
    thread::sleep(Duration::from_secs(3));
    let stream = TcpStream::connect(address).await?;
    Ok(stream)
}

async fn ping(stream: &TcpStream, i: u32) -> Result<(), std::io::Error> {
    let mut writer = BufWriter::new(stream);
    let mut rng = rand::thread_rng();
    let nonce: u32 = rng.gen();
    let ping = Ping { nonce: nonce };
    println!("Ping {} {} {}", stream.local_addr()?, i, ping.nonce);
    writer.write(&ping.nonce.to_be_bytes()).await?;
    writer.flush().await?;
    Ok(())
}
