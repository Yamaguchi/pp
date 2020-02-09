pub trait Message {}

pub struct Ping {
    pub nonce: u32,
}

impl Ping {}

impl Message for Ping {}

pub struct Pong {
    pub nonce: u32,
}

impl Message for Pong {}
