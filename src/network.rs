use rand::Rng;
use std::io::{BufWriter, Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use crate::configuration::*;
use crate::message::*;

pub struct Server {
    pub configuration: ServerConfiguration,
}

impl Server {
    pub fn start(&self) {
        println!("Start ...");
        let listener =
            TcpListener::bind(self.configuration.address.clone()).expect("failed tcp binding...");
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    if let Err(e) = self.handle_client(stream) {
                        println!("Server Error: {}", e);
                    }
                }
                Err(e) => {
                    println!("Server Error: {}", e);
                }
            }
        }
    }

    fn handle_client(&self, mut stream: TcpStream) -> Result<(), Error> {
        let mut buffer = [0; 4];
        loop {
            stream.read(&mut buffer).expect("failed");
            let mut array = [0; 4];
            array.copy_from_slice(&buffer);
            let nonce = i32::from_be_bytes(array);
            println!("Pong {:?}", nonce);
            let pong = Pong { nonce: nonce };
            stream.write(&pong.nonce.to_be_bytes())?;
            stream.flush()?;
        }
    }
}
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
                    let mut rng = rand::thread_rng();
                    loop {
                        thread::sleep(Duration::from_secs(3));
                        let mut writer = BufWriter::new(&stream);

                        let ping = Ping { nonce: rng.gen() };
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
