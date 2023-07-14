use std::{
    cell::Cell,
    sync::Arc,
    time::{Duration, Instant},
};


use anyhow::bail;
use fnv::FnvHashMap;
use rsa::RsaPrivateKey;
use servidiot_primitives::{
    chunk::{section::ChunkSection, Chunk},
    nibble_vec::NibbleVec,
    player::Gamemode,
    position::{ChunkPosition, Position},
};
use servidiot_yggdrasil::authenticate::Profile;
use tokio::net::ToSocketAddrs;

use crate::{
    connection::{listener::Listener, NewPlayer, ServerState},
    io::packet::{
        client::play::ClientPlayPacket,
        server::play::{
            ChunkBitmap, ChunkData, JoinGame, KeepAlive, NetChunk,
            PlayerPositionAndLook, ServerPlayPacket, NetChunkData,
        },
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
    /// An iterator over connected clients.
    pub fn clients(&self) -> impl Iterator<Item = &Client> {
        self.clients.values()
    }

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
                    disconnected: Cell::new(false),
                    last_keepalive_time: Cell::new(Instant::now()),
                    client_known_position: Cell::new(None),
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
    /// Has this client disconnected?
    pub disconnected: Cell<bool>,
    /// The last time we sent a keepalive.
    pub last_keepalive_time: Cell<Instant>,
    /// The position the client thinks we are at.
    pub client_known_position: Cell<Option<Position>>,
    /// Packet sender.
    pub sender: flume::Sender<ServerPlayPacket>,
    /// Packet receiver.
    pub receiver: flume::Receiver<ClientPlayPacket>,
}

impl Client {
    pub const KEEPALIVE_TIME: Duration = Duration::from_secs(15);


    /// Recieved packets iterator.
    pub fn packets(&self) -> impl Iterator<Item = ClientPlayPacket> + '_ {
        self.receiver.try_iter()
    }

    pub fn set_client_known_position(&self, position: Position) {
        self.client_known_position.set(Some(position));
    }

    pub fn get_client_known_position(&self) -> Option<Position> {
        self.client_known_position.get()
    }

    pub fn client_knows_position(&self) -> bool {
        self.client_known_position.get().is_some()
    }

    /// Set this client's position.
    pub fn set_position(&self, position: Position) -> anyhow::Result<()> {
        self.client_known_position.set(Some(position));
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

    /// Sends a keep-alive, if necessary.
    /// Returns `true` if one was sent.
    pub fn send_keepalive(&self, id: i32) -> anyhow::Result<bool> {
        if self.last_keepalive_time.get().elapsed() > Self::KEEPALIVE_TIME {
            self.last_keepalive_time.set(Instant::now());
            self.send_packet(ServerPlayPacket::KeepAlive(KeepAlive { id }))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected.get() || self.sender.is_disconnected() || self.receiver.is_disconnected()
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

    /// Convert a chunk to a network chunk.
    fn chunk_to_net(
        chunk: &Chunk,
        to_send: ChunkBitmap,
        compressed: bool,
    ) -> (NetChunkData, ChunkBitmap) {
        let mut primary_bit_map = to_send;
        for n in 0..ChunkSection::SECTIONS_PER_CHUNK {
            if chunk.get_section(n as u8).is_none() && primary_bit_map.get(n).unwrap_or(false) {
                //log::debug!("Before: {0000000000000000:b}", primary_bit_map.0);
                primary_bit_map.set(n, false);
                //log::debug!("After: {0000000000000000:b}", primary_bit_map.0);
            }
        }

        let mut block_types = vec![];
        let mut block_meta = NibbleVec::new();
        let mut block_light = NibbleVec::new();
        let mut block_sky_light = NibbleVec::new();
        let mut add_array = NibbleVec::new();
        let mut biome_array = [0; 256];

        for section in chunk.sections() {
            if !primary_bit_map.get(section.section_id as usize).unwrap() {
                continue;
            }
            //log::info!("DOING {:?}", section.section_id);
            block_types.extend_from_slice(&section.block_types);
            block_meta
                .backing_mut()
                .extend_from_slice(section.block_meta.get_backing());
            block_light
                .backing_mut()
                .extend_from_slice(section.block_light.get_backing());
            block_sky_light
                .backing_mut()
                .extend_from_slice(section.skylight.get_backing());

            for _ in 0..(block_types.len() / 2) {
                add_array.push(0);
            }
        }
        let mut n = 0;
        for z in 0..Chunk::WIDTH {
            for x in 0..Chunk::LENGTH {
                biome_array[n] = chunk.biomes()[x][z];
                n += 1;
            }
        }
        (
            NetChunkData {
                block_types,
                block_meta,
                block_light,
                block_sky_light: Some(block_sky_light),
                add_array: None, // FIXME sort out add
                biome_array: Box::new(biome_array),
                compressed,
            },
            primary_bit_map,
        )
    }

    /// Send a chunk to this client.
    pub fn send_chunk(&self, chunk: &Chunk, to_send: ChunkBitmap) -> anyhow::Result<()> {
        let (chunk_data, primary_bit_map) = Self::chunk_to_net(chunk, to_send, true);

        self.send_packet(ServerPlayPacket::ChunkData(ChunkData {
            chunk_x: chunk.position().x,
            chunk_z: chunk.position().z,
            ground_up_continuous: true,
            primary_bit_map,
            add_bit_map: ChunkBitmap::empty(),
            chunk_data: NetChunk::Present(chunk_data),
        }))
    }

    /// Unload a chunk for this client.
    pub fn unload_chunk(&self, position: ChunkPosition) -> anyhow::Result<()> {
        self.send_packet(ServerPlayPacket::ChunkData(ChunkData {
            chunk_x: position.x,
            chunk_z: position.z,
            ground_up_continuous: true,
            primary_bit_map: ChunkBitmap::empty(),
            add_bit_map: ChunkBitmap::empty(),
            chunk_data: NetChunk::NotPresent,
        }))
    }

    fn send_packet(&self, p: ServerPlayPacket) -> anyhow::Result<()> {
        self.sender.send(p)?;
        Ok(())
    }
}
