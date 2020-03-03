use crate::errors::Error;
use byteorder::ReadBytesExt;
use byteorder::{NetworkEndian, WriteBytesExt};
use rand::Rng;
use tokio::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub enum Message {
    RequestPing,
    RequestData(Vec<u8>),
    RequestSubscribe(Sender<Message>),
    Ping(u32),
    Pong(u32),
    Data(Vec<u8>),
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
                v.write_u16::<NetworkEndian>(self.to_type_id()).unwrap();
                v.write_u32::<NetworkEndian>(*nonce).unwrap();
            }
            Message::Data(d) => {
                v.write_u16::<NetworkEndian>(self.to_type_id()).unwrap();
                v.extend(d);
            }
            _ => {}
        }
        v
    }

    fn to_type_id(&self) -> u16 {
        match self {
            Message::Ping(_) => 0x01,
            Message::Pong(_) => 0x02,
            Message::Data(_) => 0x03,
            Message::RequestPing => 0x11,
            Message::RequestData(_) => 0x12,
            Message::RequestSubscribe(_) => 0x13,
        }
    }

    pub fn is_p2p(&self) -> bool {
        match self {
            Message::Ping(_) => true,
            Message::Pong(_) => true,
            Message::Data(_) => true,
            Message::RequestPing => false,
            Message::RequestData(_) => false,
            Message::RequestSubscribe(_) => false,
        }
    }

    pub fn parse(buffer: &[u8]) -> Result<Message, Error> {
        let (mut type_buf, mut body_buf) = buffer.split_at(2);
        let type_id = type_buf
            .read_u16::<NetworkEndian>()
            .map_err(|e| Error::CannotRead(e))?;
        let message = match type_id {
            0x01 => {
                let nonce = body_buf
                    .read_u32::<NetworkEndian>()
                    .map_err(|e| Error::CannotRead(e))?;
                Message::Ping(nonce)
            }
            0x02 => {
                let nonce = body_buf
                    .read_u32::<NetworkEndian>()
                    .map_err(|e| Error::CannotRead(e))?;
                Message::Pong(nonce)
            }
            0x03 => {
                let data = Vec::from(body_buf);
                Message::Data(data)
            }
            0x11 => Message::RequestPing,
            0x12 => {
                let data = Vec::from(body_buf);
                Message::RequestData(data)
            }
            0x13 => return Err(Error::UnsupportedOperation),
            _ => return Err(Error::UnknownMessage),
        };
        Ok(message)
    }
}
