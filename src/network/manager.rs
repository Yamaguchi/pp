use crate::application::Application;
use crate::configuration;
use crate::errors::Error;
use crate::event::{Event, EventType};
use crate::network::client::Client;
use crate::node::{add_connection, add_peer, remove_connection};
use std::net::SocketAddr;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time;

pub struct Manager {}

impl Manager {
    pub fn start<A>(app: Arc<RwLock<A>>, config: configuration::Network)
    where
        A: Application + 'static + Send + Sync,
    {
        Self::init_event(Arc::clone(&app));

        tokio::spawn(async move {
            // wait 60 secs, so that peers can start completely.
            let mut interval = time::interval(Duration::from_secs(60));
            interval.tick().await;
            let _ = Manager::connect_on_launch(app, config).await;
        });
    }

    fn init_event<A>(app: Arc<RwLock<A>>)
    where
        A: Application + 'static + Send + Sync,
    {
        let guard_app = app.read().unwrap();
        let app_ref = guard_app.deref();
        let mut guard_event = app_ref.event_manager().unwrap();
        let event_manager = guard_event.deref_mut();
        let rx = event_manager.subscribe(EventType::Disconnected).unwrap();
        match rx.recv() {
            Ok(Event::Disconnected(addr)) => {
                Self::reconnect(Arc::clone(&app), addr);
            }
            _ => {}
        }
    }

    /// errors:
    ///     Error::PeerAlreadyConnected
    ///     Error::CannotConnectPeer
    ///     Error::AuthenticationFailed
    ///     Error::CannotGetLock
    async fn connect<A>(app: Arc<RwLock<A>>, addr: SocketAddr) -> Result<(), Error>
    where
        A: Application + 'static + Send + Sync,
    {
        let peer = add_peer(Arc::clone(&app), addr)?;
        let key = {
            let guard_app = app.read().map_err(|_| Error::CannotGetLock)?;
            let app = guard_app.deref();
            app.private_key()
        };
        let client = Client::connect(peer.addr, key).await?;
        add_connection(Arc::clone(&app), client)?;
        Ok(())
    }

    async fn connect_on_launch<A>(
        app: Arc<RwLock<A>>,
        config: configuration::Network,
    ) -> Result<(), Error>
    where
        A: Application + 'static + Send + Sync,
    {
        for addr in config.connect_to {
            let cloned = Arc::clone(&app);
            Manager::connect(cloned, addr).await?;
        }
        Ok(())
    }

    fn disconnect<A>(app: Arc<RwLock<A>>, addr: SocketAddr)
    where
        A: Application + 'static + Send + Sync,
    {
        remove_connection(Arc::clone(&app), addr).ok();
    }

    fn reconnect<A>(app: Arc<RwLock<A>>, addr: SocketAddr)
    where
        A: Application + 'static + Send + Sync,
    {
        Self::disconnect(Arc::clone(&app), addr);
        tokio::spawn(async move {
            Self::connect(Arc::clone(&app), addr).await.ok();
        });
    }
}
