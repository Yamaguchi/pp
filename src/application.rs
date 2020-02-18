use crate::errors::Error;
use crate::node::Node;
use std::sync::Arc;
use std::sync::Mutex;

pub trait Application {
    fn node(&self) -> Result<std::sync::MutexGuard<'_, Node>, Error>;
}

pub struct NetworkApplication {
    node: Arc<Mutex<Node>>,
}

impl Application for NetworkApplication {
    fn node(&self) -> Result<std::sync::MutexGuard<'_, Node>, Error> {
        self.node.lock().map_err(|_| Error::CannotGetLock)
    }
}

impl NetworkApplication {
    pub fn new() -> Self {
        NetworkApplication {
            node: Arc::new(Mutex::new(Node::new())),
        }
    }
    async fn run(&mut self) {}
}
