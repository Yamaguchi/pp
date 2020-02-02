
pub trait Message {}


pub struct Ping {
    pub nonce: i32,
}

impl Ping {
    
}

impl Message for Ping {
}

pub struct Pong {
    pub nonce: i32,
}

impl Message for Pong {

}