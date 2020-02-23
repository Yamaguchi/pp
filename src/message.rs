use byteorder::{NetworkEndian, WriteBytesExt};
use rand::Rng;

pub trait Message {
    fn to_bytes(&self) -> Vec<u8>;
}

pub struct Ping {
    pub nonce: u32,
}

impl Ping {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let nonce: u32 = rng.gen();
        Ping { nonce: nonce }
    }
}

impl Message for Ping {
    fn to_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.write_u32::<NetworkEndian>(self.nonce).unwrap();
        v
    }
}

pub struct Pong {
    pub nonce: u32,
}

impl Message for Pong {
    fn to_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.write_u32::<NetworkEndian>(self.nonce).unwrap();
        v
    }
}
