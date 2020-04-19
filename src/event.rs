use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PublicKey;
use std::collections::HashMap;
use std::hash::Hash;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref EVENT_MANAGER: Arc<Mutex<EventManager>> =
        { Arc::new(Mutex::new(EventManager::new())) };
}

pub struct EventManager {
    senders: HashMap<EventType, Vec<SyncSender<Event>>>,
}

impl EventManager {
    fn new() -> Self {
        EventManager {
            senders: HashMap::new(),
        }
    }

    pub fn subscribe(event_type: EventType) -> Result<Receiver<Event>, Error> {
        let mut guard = EVENT_MANAGER.lock().map_err(|_| Error::CannotGetLock)?;
        let m = guard.deref_mut();
        m.subscribe_inner(event_type)
    }

    fn subscribe_inner(&mut self, event_type: EventType) -> Result<Receiver<Event>, Error> {
        let (tx, rx) = sync_channel(1);
        if self.senders.contains_key(&event_type) {
            let mut senders = self.senders[&event_type].to_vec();
            senders.push(tx);
            self.senders.insert(event_type, senders);
        } else {
            let vec = vec![tx];
            self.senders.insert(event_type, vec);
        }
        Ok(rx)
    }

    pub fn broadcast(event: Event) -> Result<(), Error> {
        let mut guard = EVENT_MANAGER.lock().map_err(|_| Error::CannotGetLock)?;
        let m = guard.deref_mut();
        m.broadcast_inner(event)
    }

    fn broadcast_inner(&mut self, event: Event) -> Result<(), Error> {
        info!("EventManager#broadcast");
        let event_type = event.to_type();
        for s in &self.senders[&event_type] {
            let _ = s.send(event.clone());
        }
        Ok(())
    }
}

#[derive(Hash, Eq, PartialEq)]
pub enum EventType {
    Connected,
    Authenticated,
    Disconnected,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Event {
    Connected(SocketAddr),
    Authenticated(PublicKey<Ed25519>),
    Disconnected(SocketAddr),
}

impl Event {
    fn to_type(&self) -> EventType {
        match self {
            Event::Connected(_) => EventType::Connected,
            Event::Authenticated(_) => EventType::Authenticated,
            Event::Disconnected(_) => EventType::Disconnected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_and_broadcast() {
        if let Ok(rx) = EventManager::subscribe(EventType::Connected) {
            let e = Event::Connected("[::1]:1000".parse().unwrap());
            let _ = EventManager::broadcast(e.clone());
            assert_eq!(e, rx.recv().unwrap());
        } else {
            panic!("subscribe failed.");
        }
    }
}
