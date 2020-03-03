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
        if buffer.len() < 18 {
            rest.copy_from_slice(buffer);
            return Ok((None, rest));
        }
        let mut decrypted = [0u8; 18];
        state
            .read_message(&buffer[0..18], &mut decrypted)
            .map_err(|e| Error::TransportError(e))?;
        let mut c = Cursor::new(&decrypted);
        let body_len: u16 = c
            .read_u16::<NetworkEndian>()
            .map_err(|e| Error::CannotRead(e))?;
        let len: usize = (2 + 16 + body_len + 16) as usize;
        if buffer.len() < len {
            rest.copy_from_slice(buffer);
            return Ok((None, rest));
        }

        let mut decrypted = [0u8; 65535];
        let n = state
            .read_message(&buffer[18..len], &mut decrypted)
            .map_err(|e| Error::TransportError(e))?;
        let message = Message::parse(&decrypted[..n])?;
        rest.extend(&buffer[len..]);
        Ok((Some(message), rest))
    }
    pub fn write_message(state: &mut TransportState, message: Message) -> Result<Vec<u8>, Error> {
        let mut buf = vec![];
        let payload = message.to_bytes();
        let mut enc_len = [0u8; 18];
        let mut len_buf = vec![];
        len_buf
            .write_u16::<NetworkEndian>(payload.len() as u16)
            .map_err(|e| Error::CannotWrite(e))?;
        let len_len = state
            .write_message(&len_buf[..], &mut enc_len[..])
            .map_err(|e| Error::TransportError(e))?;
        buf.write(&enc_len[..len_len])
            .map_err(|e| Error::CannotWrite(e))?;
        let mut enc_payload = [0u8; 65535];
        let len_payload = state
            .write_message(&payload, &mut enc_payload[..])
            .map_err(|e| Error::TransportError(e))?;
        buf.write(&enc_payload[..len_payload])
            .map_err(|e| Error::CannotWrite(e))?;
        Ok(buf)
    }
}
