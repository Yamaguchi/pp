use crate::application::Application;
use network::initiate_response::Event;
use network::network_service_server::{NetworkService, NetworkServiceServer};
use network::{AlreadyConnected, Authenticated, Connected, Disconnected};
use network::{InitiateRequest, InitiateResponse};
use std::net::SocketAddr;
use std::net::SocketAddrV6;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};

pub struct GrpcServer<A>
where
    A: Application + 'static + Send + Sync,
{
    app: Arc<RwLock<A>>,
    address: String,
}

impl<A> GrpcServer<A>
where
    A: Application + 'static + Send + Sync,
{
    pub fn new(app: Arc<RwLock<A>>, address: String) -> Self {
        GrpcServer::<A> {
            app: app,
            address: address,
        }
    }

    pub async fn start(&self) {
        let addr: SocketAddr = self.address.parse().unwrap();

        let app = Arc::clone(&self.app);
        tokio::spawn(async move {
            let service = NetworkServiceImpl { app: app };
            let _ = Server::builder()
                .add_service(NetworkServiceServer::new(service))
                .serve(addr.clone())
                .await;
        });
    }
}

pub mod network {
    tonic::include_proto!("network.grpc");
}

#[derive(Clone)]
struct NetworkServiceImpl<A>
where
    A: Application + 'static + Send + Sync,
{
    app: Arc<RwLock<A>>,
}

impl<A> NetworkServiceImpl<A> where A: Application + 'static + Send + Sync {}

#[tonic::async_trait]
impl<A> NetworkService for NetworkServiceImpl<A>
where
    A: Application + 'static + Send + Sync,
{
    type InitiateStream = mpsc::Receiver<Result<InitiateResponse, Status>>;
    async fn initiate(
        &self,
        request: Request<InitiateRequest>,
    ) -> Result<Response<Self::InitiateStream>, Status> {
        let (mut tx, rx) = mpsc::channel(1);

        let host = request.get_ref().host.clone();
        let port = request.get_ref().port;
        let (sender, mut receiver) = mpsc::channel(1);

        let cloned = Arc::clone(&self.app);
        tokio::spawn(async move {
            {
                let mut guard = cloned.read().unwrap();
                let app = guard.deref();
                // let app = guard;
                let addr: SocketAddrV6 = format!("{}:{}", host, port)
                    .parse()
                    .expect("cannot parse address");
                // let mut n = ;
                app.node()
                    .ok()
                    .unwrap()
                    .deref_mut()
                    .connect(SocketAddr::V6(addr), sender);
            }
            // if let Ok(mut n) = app.node() {
            //     n.connect(SocketAddr::V6(addr), sender);
            // }
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
