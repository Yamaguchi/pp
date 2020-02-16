use std::env;

use crate::application::Application;
use crate::configuration::*;
use crate::errors::Error;
use crate::grpc::GrpcServer;
use crate::network::server::Server;
use crate::node::Node;
use std::sync::Arc;
use std::sync::Mutex;

mod application;
mod configuration;
mod crypto;
mod errors;
mod grpc;
mod key;
mod message;
mod network;
mod node;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let bind_address = &args[1];
    let rpc_address = &args[2];

    env::set_var("RUST_LOG", "trace");
    env_logger::init();

    let app = Arc::new(Mutex::new(NetworkApplication::new()));
    let cloned = Arc::clone(&app);
    let rpc = GrpcServer::new(cloned, rpc_address.clone());
    rpc.start().await;

    let config = ServerConfiguration {
        address: bind_address.clone().to_string(),
    };
    let cloned = Arc::clone(&app);
    let mut server = Server::new(cloned, config);
    server.start().await;
}

struct NetworkApplication {
    node: Arc<Mutex<Node>>,
}

impl Application for NetworkApplication {
    fn node(&self) -> Result<std::sync::MutexGuard<'_, node::Node>, Error> {
        self.node.lock().map_err(|_| Error::CannotGetLock)
    }
}

impl NetworkApplication {
    fn new() -> Self {
        NetworkApplication {
            node: Arc::new(Mutex::new(Node::new())),
        }
    }
    async fn run(&mut self) {}
}
