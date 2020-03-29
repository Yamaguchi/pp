use crate::application::NetworkApplication;
use crate::configuration::*;
use crate::grpc::GrpcServer;
use crate::network::manager::Manager;
use crate::network::server::Server;
use std::env;
use std::sync::Arc;
use std::sync::RwLock;

mod application;
mod configuration;
mod crypto;
mod errors;
mod event;
mod grpc;
mod key;
mod message;
mod network;
mod node;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate toml;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];
    let config = Configuration::new(path.to_string()).expect("can not read config file");

    env::set_var("RUST_LOG", "trace");
    env_logger::init();

    let app = Arc::new(RwLock::new(NetworkApplication::new(config.application)));
    let cloned = Arc::clone(&app);
    let rpc = GrpcServer::new(cloned, config.grpc);
    rpc.start().await;

    let cloned = Arc::clone(&app);
    Manager::start(cloned, config.network);

    let cloned = Arc::clone(&app);
    let mut server = Server::new(cloned, config.server);
    server.start().await;
}
