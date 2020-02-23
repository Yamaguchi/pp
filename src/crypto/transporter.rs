use crate::errors::Error;
use crate::message::Message;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use snow::TransportState;
use std::io::Cursor;
use std::io::Write;

pub struct Transporter;

impl Transporter {
    pub fn read_message(
        state: &mut TransportState,
        buffer: &[u8],
    ) -> Result<(Option<Message>, Vec<u8>), Error> {
        let mut rest: Vec<u8> = vec![];
        if buffer.len() < 16 {
            rest.copy_from_slice(buffer);
            return Ok((None, rest));
        }
        let mut decrypted = [0u8; 16];
        state
            .read_message(buffer, &mut decrypted)
            .map_err(|_| Error::TransportError)?;
        let mut c = Cursor::new(&decrypted);
        let body_len: u16 = c.read_u16::<NetworkEndian>().unwrap();
        let len: usize = (body_len + 16) as usize;
        if buffer.len() < len {
            rest.copy_from_slice(buffer);
            return Ok((None, rest));
        }

        let mut decrypted = [0u8; 65535];
        state
            .read_message(&buffer[16..len], &mut decrypted)
            .map_err(|_| Error::TransportError)?;
        let message = Message::parse(&decrypted)?;
        rest.copy_from_slice(&buffer[len..]);
        Ok((Some(message), rest))
    }
    pub fn write_message(
        state: &mut TransportState,
        buffer: &mut [u8; 65535],
        message: Message,
    ) -> Result<usize, Error> {
        let payload = message.to_bytes();
        let mut enc_len = [0u8; 16];
        let mut len_buf = vec![];
        len_buf
            .write_u16::<NetworkEndian>(payload.len() as u16)
            .unwrap();
        let len_len = state
            .write_message(&len_buf[..], &mut enc_len[..])
            .map_err(|_| Error::TransportError)?;
        let mut vec = vec![];
        vec.write(&enc_len[..len_len]).unwrap();

        let mut enc_payload = [0u8; 65535];
        let len_payload = state
            .write_message(&payload, &mut enc_payload[..])
            .map_err(|_| Error::TransportError)?;
        vec.write(&enc_payload[..len_payload]);
        buffer.copy_from_slice(&vec[..]);
        Ok(vec.len())
    }
}
