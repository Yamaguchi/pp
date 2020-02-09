use std::env;

use crate::configuration::*;
use crate::network::client::Client;
use crate::network::server::Server;

mod configuration;
mod message;
mod network;

fn main() {
    let args: Vec<String> = env::args().collect();

    let to_address = &args[2];
    let client = Client {
        configuration: ClientConfiguration {
            address: to_address.to_string(),
        },
    };
    client.start();

    let bind_address = &args[1];
    let server = Server {
        configuration: ServerConfiguration {
            address: bind_address.clone().to_string(),
        },
    };
    server.start();
}
