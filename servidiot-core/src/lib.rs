use std::{
    net::SocketAddr,
    num::{NonZeroU64, NonZeroUsize},
    sync::Arc,
};

use game::GameState;
use servidiot_utils::ticks::TickLoop;
use thiserror::Error;
use tokio::io;

mod game;
mod systems;
mod entity;
mod world;
mod events;


pub struct Config {
    pub net_threads: NonZeroUsize,
    pub game_threads: NonZeroUsize,
    pub tps: NonZeroU64,
    pub bind_addr: SocketAddr,
}

/// Represents the game runtime.
pub struct GameRuntime {
    state: GameState,
    net_runtime: tokio::runtime::Runtime,
    config: Arc<Config>,
}

impl GameRuntime {
    /// Sets up the game runtime.
    pub fn create(config: Arc<Config>) -> std::result::Result<Self, RuntimeCreationError> {
        let net_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(config.net_threads.get())
            .enable_all()
            .build()?;

        let game_state = GameState::create(config.clone(), net_runtime.handle());

        Ok(Self {
            state: game_state?,
            net_runtime,
            config,
        })
    }

    /// Begins running the runtime.
    pub fn run(self) {
        TickLoop::new(self.config.tps, || {
            self.state.systems().borrow().run_systems(&self.state);

            true
        }).run();
    }
}

#[derive(Error, Debug)]
pub enum RuntimeCreationError {
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
    #[error("Error: {0}")]
    GenericError(#[from] anyhow::Error),
}
