use crate::errors::Error;
use crate::node::Node;

pub trait Application {
    fn node(&self) -> Result<std::sync::MutexGuard<'_, Node>, Error>;
}
