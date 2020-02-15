use rand::Rng;

pub trait Message {}

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

impl Message for Ping {}

pub struct Pong {
    pub nonce: u32,
}

impl Message for Pong {}
