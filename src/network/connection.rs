use crate::errors::Error;
use async_trait::async_trait;
use std::marker::Sized;

#[async_trait]
pub trait Connection: Sized {
    async fn write(&mut self, buf: &[u8]) -> Result<(), Error>;
    async fn read(&mut self) -> Result<Vec<u8>, Error>;
    async fn shutdown(&mut self) -> Result<(), Error>;
}
