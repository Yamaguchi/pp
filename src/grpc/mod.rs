use crate::node::context::NODE;

use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};

use network::initiate_response::Event;
use network::network_service_server::{NetworkService, NetworkServiceServer};
use network::{AlreadyConnected, Authenticated, Connected, Disconnected};
use network::{InitiateRequest, InitiateResponse};
use std::net::SocketAddr;
use std::net::SocketAddrV6;
pub struct GrpcServer {
    address: String,
}

impl GrpcServer {
    pub fn new(address: String) -> Self {
        GrpcServer { address: address }
    }

    pub async fn start(&self) {
        // if let Ok(mut rt) = Runtime::new() {
        let addr: SocketAddr = self.address.parse().unwrap();

        tokio::spawn(async move {
            let service = NetworkServiceImpl {};
            let _ = Server::builder()
                .add_service(NetworkServiceServer::new(service))
                .serve(addr.clone())
                .await;
        });
        // }
    }
}

pub mod network {
    tonic::include_proto!("network.grpc");
}

#[derive(Clone)]
struct NetworkServiceImpl {}

impl NetworkServiceImpl {}
#[tonic::async_trait]
impl NetworkService for NetworkServiceImpl {
    type InitiateStream = mpsc::Receiver<Result<InitiateResponse, Status>>;
    async fn initiate(
        &self,
        request: Request<InitiateRequest>,
    ) -> Result<Response<Self::InitiateStream>, Status> {
        let (mut tx, rx) = mpsc::channel(1);

        let host = request.get_ref().host.clone();
        let port = request.get_ref().port;
        let (sender, mut receiver) = mpsc::channel(1);

        tokio::spawn(async move {
            match NODE.write() {
                Ok(mut n) => {
                    let addr: SocketAddrV6 = format!("{}:{}", host, port)
                        .parse()
                        .expect("cannot parse address");
                    n.connect(SocketAddr::V6(addr), sender);
                }
                Err(_) => {}
            };
            while let Some(res) = receiver.recv().await {
                let response = InitiateResponse {
                    event: Some(res.clone()),
                };
                let _ = tx.send(Ok(response)).await;
                match res {
                    Event::Connected(Connected { public_key }) => {
                        info!("Connected {}", public_key);
                    }
                    Event::Disconnected(Disconnected { public_key }) => {
                        info!("Disconnected {}", public_key);
                        break;
                    }
                    Event::AlreadyConnected(AlreadyConnected { public_key }) => {
                        info!("AlreadyConnected {}", public_key);
                        break;
                    }
                    Event::Authenticated(Authenticated {
                        public_key,
                        remote_public_key,
                    }) => {
                        info!("Authenticated {}, {}", public_key, remote_public_key);
                        break;
                    }
                    _ => {}
                }
            }
        });
        Ok(Response::<Self::InitiateStream>::new(rx))
    }
}
