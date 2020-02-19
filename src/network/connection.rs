use crate::errors::Error;
use crate::node::Connections;
use async_trait::async_trait;
use bytes::BytesMut;
use std::marker::Sized;

#[async_trait]
pub trait Connection: Sized {
    async fn write(&mut self, buf: &[u8]) -> Result<(), Error>;
    async fn read(&mut self, mut buf: BytesMut) -> Result<usize, Error>;
    async fn shutdown(&mut self) -> Result<(), Error>;
}
