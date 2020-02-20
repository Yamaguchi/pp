use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PrivateKey;
use crate::node::Node;
use std::sync::Arc;
use std::sync::Mutex;
pub trait Application {
    fn node(&self) -> Result<std::sync::MutexGuard<'_, Node>, Error>;
    fn private_key(&self) -> PrivateKey<Ed25519>;
}

pub struct NetworkApplication {
    node: Arc<Mutex<Node>>,
    private_key: PrivateKey<Ed25519>,
}

impl Application for NetworkApplication {
    fn node(&self) -> Result<std::sync::MutexGuard<'_, Node>, Error> {
        self.node.lock().map_err(|_| Error::CannotGetLock)
    }
    fn private_key(&self) -> PrivateKey<Ed25519> {
        self.private_key.clone()
    }
}

impl NetworkApplication {
    pub fn new() -> Self {
        let random_bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        NetworkApplication {
            node: Arc::new(Mutex::new(Node::new())),
            private_key: PrivateKey::<Ed25519>::new(&random_bytes),
        }
    }
    async fn run(&mut self) {}
}
