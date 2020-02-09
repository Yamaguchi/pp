use rand::Rng;
use std::io::{BufWriter, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use crate::configuration::*;
use crate::message::*;

pub struct Client {
    pub configuration: ClientConfiguration,
}

impl Client {
    pub fn start(&self) {
        let address = self.configuration.address.clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_secs(3));
            match TcpStream::connect(address) {
                Ok(stream) => {
                    // authenticator.
                    let mut rng = rand::thread_rng();
                    loop {
                        thread::sleep(Duration::from_secs(3));
                        let mut writer = BufWriter::new(&stream);

                        let nonce: u32 = rng.gen();
                        let ping = Ping { nonce: nonce };
                        println!("Ping {}", ping.nonce);
                        if let Err(e) = writer.write(&ping.nonce.to_be_bytes()) {
                            println!("Client Error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Client Error: {}", e);
                }
            }
        });
    }
}
