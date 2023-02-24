use std::sync::Arc;

use rsa::{RsaPrivateKey};
use servidiot_yggdrasil::authenticate::Profile;

use crate::io::packet::{server::play::ServerPlayPacket, client::play::ClientPlayPacket};

pub mod listener;
pub mod worker;

/// A new player connecting to the game.
pub struct NewPlayer {
    /// This player's profile.
    pub profile: Arc<Profile>,

    /// Packet sender.
    pub sender: flume::Sender<ServerPlayPacket>,
    /// Packet receiver.
    pub receiver: flume::Receiver<ClientPlayPacket>
}

/// The server state.
pub struct ServerState {
    pub rsa_key: RsaPrivateKey
}