use std::env;

use crate::configuration::*;
use crate::network::Client;
use crate::network::Server;

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
            address: bind_address.to_string(),
        },
    };
    server.start();
}
