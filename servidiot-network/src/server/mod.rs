use std::sync::Arc;

use fnv::FnvHashMap;
use rsa::RsaPrivateKey;
use servidiot_yggdrasil::authenticate::Profile;
use tokio::net::ToSocketAddrs;

use crate::{connection::{NewPlayer, listener::Listener, ServerState}, io::packet::{server::play::ServerPlayPacket, client::play::ClientPlayPacket}};

use self::id::NetworkID;

pub mod id;

/// A minecraft network server.
pub struct Server {
    new_clients: flume::Receiver<NewPlayer>,
    clients: FnvHashMap<NetworkID, Client>,
    _state: Arc<ServerState>,
}

impl Server {
    /// Bind this server to an address.
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> anyhow::Result<Self> {
        let (send, recv) = flume::unbounded();
        let mut rng = rand::thread_rng();
        let bits = 1024;
        let server_state = ServerState {
            rsa_key: RsaPrivateKey::new(&mut rng, bits).unwrap()
        };
        let server_state = Arc::new(server_state);
        let listener = Listener::bind(addr, send, server_state.clone()).await?;
        listener.start().await;
        Ok(Self {
            new_clients: recv,
            clients: Default::default(),
            _state: server_state,
        })
    }

    /// Accept new clients.
    pub fn accept_clients(&mut self) -> Vec<NetworkID> {
        let mut ids = vec![];
        for v in self.new_clients.try_iter() {
            let id = NetworkID::new();
            self.clients.insert(id, Client { profile: v.profile, id, sender: v.sender, receiver: v.receiver });
            ids.push(id);
        }
        ids

    }
    /// Removes a client from the list.
    pub fn remove_client(&mut self, c: NetworkID) -> bool {
        self.clients.remove(&c).is_some()
    }
}

pub struct Client {
    /// This player's profile.
    pub profile: Arc<Profile>,
    /// This player's ID.
    pub id: NetworkID,

    /// Packet sender.
    pub sender: flume::Sender<ServerPlayPacket>,
    /// Packet receiver.
    pub receiver: flume::Receiver<ClientPlayPacket>
}