use std::env;

use crate::configuration::*;
use crate::grpc::GrpcServer;
use crate::network::server::Server;

mod configuration;
mod crypto;
mod errors;
mod grpc;
mod key;
mod message;
mod network;
mod node;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    env::set_var("RUST_LOG", "trace");
    env_logger::init();

    let rpc_address = &args[2];
    let rpc = GrpcServer::new(rpc_address.clone());
    rpc.start().await;

    let bind_address = &args[1];
    // let bind_address = "localhost:3000";
    let config = ServerConfiguration {
        address: bind_address.clone().to_string(),
    };
    let mut server = Server::new(config);
    server.start().await;
}
