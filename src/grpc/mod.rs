use crate::application::Application;
use crate::errors::Error;
use crate::key::PublicKey;
use crate::message::Message;
use crate::network::client::Client;
use crate::node::{add_connection, add_peer, send_to_peer};
use network::initiate_response::Event;
use network::network_service_server::{NetworkService, NetworkServiceServer};
use network::{AlreadyConnected, Connected, Disconnected};
use network::{
    InitiateRequest, InitiateResponse, RecvRequest, RecvResponse, SendRequest, SendResponse,
};
use std::net::SocketAddr;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
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

    type RecvStream = mpsc::Receiver<Result<RecvResponse, Status>>;

    async fn initiate(
        &self,
        request: Request<InitiateRequest>,
    ) -> Result<Response<Self::InitiateStream>, Status> {
        info!("initiate ...");
        let (tx, rx) = mpsc::channel(1);

        let host = request.get_ref().host.clone();
        let port = request.get_ref().port;
        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .expect("cannot parse address");

        let cloned = Arc::clone(&self.app);
        info!("app cloned ...");
        tokio::spawn(async move {
            info!("spawned ...");
            create_initiate_response(cloned, tx, addr).await;
        });
        Ok(Response::<Self::InitiateStream>::new(rx))
    }

    async fn send(&self, request: Request<SendRequest>) -> Result<Response<SendResponse>, Status> {
        info!("send ...");

        let cloned = Arc::clone(&self.app);
        let public_key = request.get_ref().public_key.clone();
        let data = request.get_ref().data.clone();
        let result = send_to_peer(
            cloned,
            Message::RequestData(hex::decode(data).unwrap()),
            &PublicKey::from_str(&public_key).unwrap(),
        );
        let response = match result {
            Ok(_) => SendResponse {
                event: Some(network::send_response::Event::Success(network::Success {})),
            },
            Err(e) => SendResponse {
                event: Some(network::send_response::Event::Error(network::Error {
                    description: format!("{:?}", e),
                })),
            },
        };
        Ok(Response::<SendResponse>::new(response))
    }

    async fn recv(
        &self,
        _request: Request<RecvRequest>,
    ) -> Result<Response<Self::RecvStream>, Status> {
        info!("recv ...");
        let (_tx, rx) = mpsc::channel(1);
        Ok(Response::<Self::RecvStream>::new(rx))
    }
}

fn already_connected() -> Event {
    Event::AlreadyConnected(AlreadyConnected {
        public_key: "".to_string(),
    })
}
fn disconnected() -> Event {
    Event::Disconnected(Disconnected {
        public_key: "".to_string(),
    })
}
fn connected() -> Event {
    Event::Connected(Connected {
        public_key: "".to_string(),
    })
}

async fn response(mut tx: Sender<Result<InitiateResponse, Status>>, e: Event) {
    let res = tx.send(Ok(InitiateResponse { event: Some(e) })).await;
    info!("result: {:?}", res)
}

async fn create_initiate_response<A>(
    app: Arc<RwLock<A>>,
    tx: Sender<Result<InitiateResponse, Status>>,
    addr: SocketAddr,
) where
    A: Application + 'static + Send + Sync,
{
    debug!("create_initiate_response ...");
    let peer = match add_peer(Arc::clone(&app), addr) {
        Ok(peer) => peer,
        Err(Error::PeerAlreadyConnected) => {
            response(tx.clone(), already_connected()).await;
            return;
        }
        _ => {
            return;
        }
    };
    let key = {
        let guard_app = app.read().unwrap();
        let app = guard_app.deref();
        app.private_key()
    };
    debug!("create_initiate_response connecting ...");
    let client = match Client::connect(peer.addr, key).await {
        Ok(client) => client,
        Err(_) => {
            response(tx.clone(), disconnected()).await;
            return;
        }
    };
    debug!("create_initiate_response connected ...");
    match add_connection(Arc::clone(&app), client) {
        Ok(_) => {
            // schedule_ping(Arc::clone(&app), peer.addr);
            response(tx.clone(), connected()).await;
        }
        Err(_) => {
            response(tx.clone(), disconnected()).await;
        }
    }
}
