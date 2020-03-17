use crate::application::Application;
use crate::configuration;
use crate::errors::Error;
use crate::network::client::Client;
use crate::node::{add_connection, add_peer};
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time;

pub struct Manager {}

impl Manager {
    pub fn start<A>(app: Arc<RwLock<A>>, config: configuration::Network)
    where
        A: Application + 'static + Send + Sync,
    {
        tokio::spawn(async move {
            // wait 60 secs, so that peers can start completely.
            let mut interval = time::interval(Duration::from_secs(60));
            interval.tick().await;
            Manager::connect_on_launch(app, config).await;
        });
    }
    /// errors:
    ///     Error::PeerAlreadyConnected
    ///     errors::Error::
    async fn connect<A>(app: Arc<RwLock<A>>, addr: SocketAddr) -> Result<(), Error>
    where
        A: Application + 'static + Send + Sync,
    {
        let peer = add_peer(Arc::clone(&app), addr)?;
        let key = {
            let guard_app = match app.read() {
                Ok(lock) => lock,
                Err(e) => {
                    error!("can not get lock {:?}", e);
                    return Ok(());
                }
            };
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
            Manager::connect(cloned, addr).await;
        }
        Ok(())
    }
}
