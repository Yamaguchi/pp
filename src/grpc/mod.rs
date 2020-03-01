use crate::application::Application;
use crate::crypto::curves::Ed25519;
use crate::errors::Error;
use crate::key::PublicKey;
use crate::message::Message;
use crate::network::client::Client;
use crate::node::{add_connection, add_peer, send_to_peer};
use network::initiate_response;
use network::initiate_response::{AlreadyConnected, Connected, Disconnected};
use network::network_service_server::{NetworkService, NetworkServiceServer};
use network::recv_response;
use network::send_response;
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
        tokio::spawn(async move {
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
                event: Some(send_response::Event::Success(send_response::Success {})),
            },
            Err(e) => SendResponse {
                event: Some(send_response::Event::Error(network::Error {
                    description: format!("{:?}", e),
                })),
            },
        };
        Ok(Response::<SendResponse>::new(response))
    }

    async fn recv(
        &self,
        request: Request<RecvRequest>,
    ) -> Result<Response<Self::RecvStream>, Status> {
        info!("recv ...");
        let (tx, rx) = mpsc::channel(1);
        let cloned = Arc::clone(&self.app);
        let hex = request.get_ref().public_key.clone();
        let public_key: PublicKey<Ed25519> = PublicKey::from_str(&hex)
            .map_err(|e| Status::invalid_argument(format!("public_key is invalid: {:?}", e)))?;
        tokio::spawn(async move {
            create_recv_response(cloned, tx, public_key).await;
        });
        Ok(Response::<Self::RecvStream>::new(rx))
    }
}

fn already_connected() -> initiate_response::Event {
    initiate_response::Event::AlreadyConnected(AlreadyConnected {
        public_key: "".to_string(),
    })
}
fn disconnected() -> initiate_response::Event {
    initiate_response::Event::Disconnected(Disconnected {
        public_key: "".to_string(),
    })
}
fn connected() -> initiate_response::Event {
    initiate_response::Event::Connected(Connected {
        public_key: "".to_string(),
    })
}
fn error(e: Error) -> recv_response::Event {
    recv_response::Event::Error(network::Error {
        description: format!("{:?}", e),
    })
}

fn success(public_key: PublicKey<Ed25519>, m: Message) -> recv_response::Event {
    recv_response::Event::Success(recv_response::Success {
        remote_public_key: public_key.to_string(),
        data: hex::encode(m.to_bytes()),
    })
}

async fn response(mut tx: Sender<Result<InitiateResponse, Status>>, e: initiate_response::Event) {
    match tx.send(Ok(InitiateResponse { event: Some(e) })).await {
        Ok(_) => {}
        Err(e) => warn!("can not send message: {:?}", e),
    }
}

async fn recv_response(mut tx: Sender<Result<RecvResponse, Status>>, e: recv_response::Event) {
    match tx.send(Ok(RecvResponse { event: Some(e) })).await {
        Ok(_) => {}
        Err(e) => warn!("can not send message: {:?}", e),
    }
}
async fn create_initiate_response<A>(
    app: Arc<RwLock<A>>,
    tx: Sender<Result<InitiateResponse, Status>>,
    addr: SocketAddr,
) where
    A: Application + 'static + Send + Sync,
{
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
    let client = match Client::connect(peer.addr, key).await {
        Ok(client) => client,
        Err(_) => {
            response(tx.clone(), disconnected()).await;
            return;
        }
    };
    match add_connection(Arc::clone(&app), client) {
        Ok(_) => {
            response(tx.clone(), connected()).await;
        }
        Err(_) => {
            response(tx.clone(), disconnected()).await;
        }
    }
}

async fn create_recv_response<A>(
    app: Arc<RwLock<A>>,
    tx: Sender<Result<RecvResponse, Status>>,
    public_key: PublicKey<Ed25519>,
) where
    A: Application + 'static + Send + Sync,
{
    let (peer_tx, mut peer_rx) = mpsc::channel::<Message>(1);
    let message = Message::RequestSubscribe(peer_tx.clone());
    match send_to_peer(Arc::clone(&app), message, &public_key.clone()) {
        Ok(()) => (),
        Err(e) => {
            recv_response(tx.clone(), error(e)).await;
            return;
        }
    }
    while let Some(m) = peer_rx.recv().await {
        recv_response(tx.clone(), success(public_key.clone(), m)).await;
    }
}
