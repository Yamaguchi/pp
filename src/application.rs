use crate::configuration;
use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PrivateKey;
use crate::node::Node;
use crate::event::EventManager;
use std::sync::Arc;
use std::sync::Mutex;
pub trait Application {
    fn node(&self) -> Result<std::sync::MutexGuard<'_, Node>, Error>;
    fn private_key(&self) -> PrivateKey<Ed25519>;
    fn event_manager(&self) -> Result<std::sync::MutexGuard<'_, EventManager>, Error>;
}

pub struct NetworkApplication {
    node: Arc<Mutex<Node>>,
    private_key: PrivateKey<Ed25519>,
    event_manager: Arc<Mutex<EventManager>>,
}

impl Application for NetworkApplication {
    fn node(&self) -> Result<std::sync::MutexGuard<'_, Node>, Error> {
        self.node.lock().map_err(|_| Error::CannotGetLock)
    }
    fn private_key(&self) -> PrivateKey<Ed25519> {
        self.private_key.clone()
    }
    fn event_manager(&self) -> Result<std::sync::MutexGuard<'_, EventManager>, Error> {
        self.event_manager.lock().map_err(|_| Error::CannotGetLock)
    }
}

impl NetworkApplication {
    pub fn new(config: configuration::Application) -> Self {
        NetworkApplication {
            node: Arc::new(Mutex::new(Node::new(config.clone()))),
            private_key: PrivateKey::<Ed25519>::new(
                &hex::decode(config.private_key).expect("cannot decode private key"),
            ),
            event_manager: Arc::new(Mutex::new(EventManager::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let config = configuration::Configuration::new("./config.sample.toml".to_string()).unwrap();
        let app = NetworkApplication::new(config.application);
        assert_eq!(
            app.private_key().to_string(),
            "649293486b0d3af1f90243021453dcb7dbbbd9fd3a54c373eaca02d230aa3154"
        );
    }
}
