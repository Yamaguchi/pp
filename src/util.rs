use crate::errors::Error;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, MutexD};

pub trait PubSub {
    fn subscribe(pattern: T) -> Result<(), Error>;
    fn broadcast(message: T) -> Result<(), Error>;
}

pub struct PubSubImpl {
    subscribers: Arc<Mutex<Receiver>>,
}
