use crate::errors::Error;
use byteorder::ReadBytesExt;
use byteorder::{NetworkEndian, WriteBytesExt};
use rand::Rng;

#[derive(Clone, Debug)]
pub enum Message {
    RequestPing,
    Ping(u32),
    Pong(u32),
}

impl Message {
    pub fn build_ping() -> Self {
        let mut rng = rand::thread_rng();
        let nonce: u32 = rng.gen();
        Message::Ping(nonce)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        match self {
            Message::Ping(nonce) | Message::Pong(nonce) => {
                v.push(self.to_type_id());
                v.write_u32::<NetworkEndian>(*nonce).unwrap();
            }
            _ => {}
        }
        v
    }
    fn to_type_id(&self) -> u8 {
        match self {
            Message::Ping(_) => 0x01,
            Message::Pong(_) => 0x02,
            Message::RequestPing => 0x11,
        }
    }

    pub fn parse(buffer: &[u8]) -> Result<Message, Error> {
        let (mut type_buf, mut body_buf) = buffer.split_at(2);
        let type_id = type_buf.read_u16::<NetworkEndian>().unwrap();
        let message = match type_id {
            0x01 => {
                let nonce = body_buf.read_u32::<NetworkEndian>().unwrap();
                Message::Ping(nonce)
            }
            0x02 => {
                let nonce = body_buf.read_u32::<NetworkEndian>().unwrap();
                Message::Ping(nonce)
            }
            0x11 => Message::RequestPing,
            _ => return Err(Error::UnknownMessage),
        };
        Ok(message)
    }
}
