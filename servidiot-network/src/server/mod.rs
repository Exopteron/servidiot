use std::sync::Arc;

use anyhow::bail;
use fnv::FnvHashMap;
use rsa::RsaPrivateKey;
use servidiot_primitives::{chunk::Chunk, player::Gamemode, position::Position, nibble_vec::NibbleVec};
use servidiot_yggdrasil::authenticate::Profile;
use tokio::net::ToSocketAddrs;

use crate::{
    connection::{listener::Listener, NewPlayer, ServerState},
    io::packet::{
        client::play::ClientPlayPacket,
        server::play::{ChunkData, JoinGame, NetChunk, PlayerPositionAndLook, ServerPlayPacket},
    },
};

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
            rsa_key: RsaPrivateKey::new(&mut rng, bits).unwrap(),
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
            self.clients.insert(
                id,
                Client {
                    profile: v.profile,
                    id,
                    sender: v.sender,
                    receiver: v.receiver,
                },
            );
            ids.push(id);
        }
        ids
    }
    /// Removes a client from the list.
    pub fn remove_client(&mut self, c: NetworkID) -> bool {
        self.clients.remove(&c).is_some()
    }

    /// Retrieves a client.
    pub fn get_client(&self, c: NetworkID) -> anyhow::Result<&Client> {
        if let Some(c) = self.clients.get(&c) {
            Ok(c)
        } else {
            bail!("client {:?} not present", c)
        }
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
    pub receiver: flume::Receiver<ClientPlayPacket>,
}

impl Client {
    /// Set this client's position.
    pub fn set_position(&self, position: Position) -> anyhow::Result<()> {
        self.send_packet(ServerPlayPacket::PlayerPositionAndLook(
            PlayerPositionAndLook {
                x: position.x,
                y: position.y,
                z: position.z,
                yaw: position.yaw,
                pitch: position.pitch,
                on_ground: position.on_ground,
            },
        ))
    }

    /// Send the Join Game message to this player.
    pub fn join_game(
        &self,
        gamemode: Gamemode,
        dimension: i8,
        difficulty: u8,
        max_players: u8,
        level_type: String,
    ) -> anyhow::Result<()> {
        self.send_packet(ServerPlayPacket::JoinGame(JoinGame {
            entity_id: self.id.0,
            gamemode,
            dimension,
            difficulty,
            max_players,
            level_type,
        }))
    }

    /// Send a full chunk to this client.
    pub fn send_chunk(&self, chunk: &Chunk) -> anyhow::Result<()> {
        let mut primary_bit_map = 0u16;
        for n in 0..16 {
            if chunk.get_section(n).is_some() {
                primary_bit_map |= 1 << n;
            }
        }

        let mut block_types = vec![];
        let mut block_meta = NibbleVec::new();
        let mut block_light = NibbleVec::new();
        let mut block_sky_light = NibbleVec::new();
        let mut add_array = NibbleVec::new();
        let mut biome_array = [0; 256];
        
        for section in chunk.sections() {
            block_types.extend_from_slice(&section.block_types);
            block_meta.backing_mut().extend_from_slice(section.block_meta.get_backing());
            block_light.backing_mut().extend_from_slice(section.block_light.get_backing());
            block_sky_light.backing_mut().extend_from_slice(section.skylight.get_backing());

            for _ in 0..(block_types.len() / 2) {
                add_array.push(0);
            }

            let mut n = 0;
            for z in 0..Chunk::WIDTH {
                for x in 0..Chunk::LENGTH {
                    biome_array[n] = chunk.biomes()[x][z];
                    n += 1;
                }
            }
        }


        self.send_packet(ServerPlayPacket::ChunkData(ChunkData {
            chunk_x: chunk.position().x,
            chunk_z: chunk.position().z,
            ground_up_continuous: true,
            primary_bit_map,
            add_bit_map: 0,
            chunk_data: NetChunk {
                block_types,
                block_meta,
                block_light,
                block_sky_light: Some(block_sky_light),
                add_array,
                biome_array: Box::new(biome_array),
            },
        }))
    }

    fn send_packet(&self, p: ServerPlayPacket) -> anyhow::Result<()> {
        self.sender.send(p)?;
        Ok(())
    }
}
