use crate::errors::Error;
use std::marker::Sized;

pub trait Connection: Sized {
    fn write(&self, buf: &[u8]) -> Result<(), Error>;
    fn read(&self) -> Result<(), Error>;
    fn shutdown(&self) -> Result<(), Error>;
}
