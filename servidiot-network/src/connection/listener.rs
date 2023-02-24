use std::sync::Arc;

use tokio::net::{TcpListener, ToSocketAddrs};

use super::{NewPlayer, ServerState, worker::Worker};

/// A Minecraft connection listener.
pub struct Listener {
    /// Our TCP listener.
    listener: TcpListener,
    /// Channel to notify the server of 
    /// new player connections.
    new_players: flume::Sender<NewPlayer>,
    /// Server state.
    server_state: Arc<ServerState>
}

impl Listener {
    /// Bind to an address.
    pub async fn bind<A: ToSocketAddrs>(address: A, new_players: flume::Sender<NewPlayer>, server_state: Arc<ServerState>) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(address).await?;
        Ok(Self {
            listener,
            new_players,
            server_state
        })
    }   

    /// Begin this listener.
    pub async fn start(self) {
        tokio::spawn(async move {
            self.run().await;
        });
    }

    /// Internal run method.
    async fn run(self) {
        loop {
            println!("rannin");
            if self.new_players.is_disconnected() {
                println!("bak");
                break;
            }
            match self.listener.accept().await {
                Ok((connection, addr)) => {
                    log::info!("Got connection from {:?}", addr);
                    let worker = Worker::new(connection, addr, self.server_state.clone(), self.new_players.clone());
                    tokio::spawn(async move {
                        if let Err(e) = worker.run().await {
                            log::error!("connection error for {:?}: {:?}", addr, e);
                        }
                    });
                },
                Err(e) => log::error!("Connection error: {:?}", e)
            }
        }
    }
}