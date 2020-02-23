use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PublicKey;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};

pub struct EventManager {
    senders: HashMap<EventType, Vec<Sender<Event>>>,
}

impl EventManager {
    pub fn new() -> Self {
        EventManager {
            senders: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, event_type: EventType) -> Result<Receiver<Event>, Error> {
        let (tx, rx) = channel();
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

    pub fn broadcast(&mut self, event: Event) -> Result<(), Error> {
        let event_type = event.to_type();
        for s in &self.senders[&event_type] {
            s.send(event.clone());
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
    Connected(i32),
    Authenticated(u32),
    Disconnected(u32),
}

impl Event {
    fn to_type(&self) -> EventType {
        match self {
            Connected => EventType::Connected,
            Authenticated => EventType::Authenticated,
            Disconnected => EventType::Disconnected,
        }
    }
}

mod tests {
    use super::*;
    #[test]
    fn test_subscribe_and_broadcast() {
        let mut m = EventManager::new();
        if let Ok(rx) = m.subscribe(EventType::Connected) {
            let e = Event::Connected(1);
            m.broadcast(e.clone());
            assert_eq!(e, rx.recv().unwrap());
        } else {
            panic!("subscribe failed.");
        }
    }
}