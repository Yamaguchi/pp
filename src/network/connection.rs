use crate::errors::Error;
use crate::node::Connections;
use async_trait::async_trait;
use std::marker::Sized;

#[async_trait]
pub trait Connection: Sized {
    async fn write(&self, buf: &[u8]) -> Result<(), Error>;
    async fn read(&self, mut buf: &[u8]) -> Result<usize, Error>;
    async fn shutdown(&self) -> Result<(), Error>;
}
