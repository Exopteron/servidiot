use std::{sync::Arc, cell::RefCell, rc::Rc, path::PathBuf, str::FromStr, collections::HashMap};

use servidiot_ecs::{World, SystemExecutor, Entity, EntityRef};
use servidiot_network::server::{id::NetworkID, Client, Server};
use servidiot_primitives::position::{ChunkLocation, ChunkPosition, EntityLocation, Location};
use servidiot_utils::{resources::Resources, events::EventManager};
use tokio::runtime::Handle;

use crate::{Config, systems, world::{view::View, GameWorld}, entity::{EntityDispatch, player::PlayerMarker}};

#[derive(Default)]
pub struct ClientMap(HashMap<NetworkID, Entity>);

impl ClientMap {
    pub fn add_mapping(&mut self, a: NetworkID, b: Entity) {
        self.0.insert(a, b);
    }

    pub fn get_mapping(&self, a: NetworkID) -> Entity {
        self.0[&a]
    }

    pub fn remove_mapping(&mut self, a: NetworkID) {
        self.0.remove(&a);
    }
}

pub struct GameState {
    ecs: RefCell<servidiot_ecs::World>,
    events: RefCell<EventManager<GameState>>,
    systems: RefCell<SystemExecutor<GameState>>,
    resources: Rc<Resources>
}

impl GameState {
    pub fn create(cfg: Arc<Config>, net_runtime: &Handle) -> anyhow::Result<Self> {
        let ecs = RefCell::new(World::new());
        let mut resources = Resources::new();

        let events = EventManager::<GameState>::new();

        let mut systems = SystemExecutor::<GameState>::new();

        systems::login::register_systems(&mut systems);
        systems::world::register_systems(&mut systems);
        systems::packet::register_systems(&mut systems);
        systems::entity::register_systems(&mut systems);
        

        resources.add(ClientMap::default());        
        resources.add(GameWorld::new(PathBuf::from_str("").unwrap()));
        resources.add(net_runtime.block_on(Server::bind(cfg.bind_addr))?);
        Ok(Self {
            ecs,
            events: RefCell::new(events),
            systems: RefCell::new(systems),
            resources: Rc::new(resources)
        })
    } 

    pub fn resources(&self) -> &Rc<Resources> {
        &self.resources
    }

    pub fn events(&self) -> &RefCell<EventManager<GameState>> {
        &self.events
    }

    pub fn systems(&self) -> &RefCell<SystemExecutor<GameState>> {
        &self.systems
    }

    pub fn ecs(&self) -> &RefCell<servidiot_ecs::World> {
        &self.ecs
    }


    pub fn load_entities_around(&self, ecs: &World, server: &Server, world: &GameWorld, this: EntityRef, dim: Location, loc: impl Iterator<Item = ChunkPosition>) -> anyhow::Result<()> {
        let us_to_unload = vec![];

        let this_id = *this.get::<&NetworkID>().unwrap();

        let we_are_player = this.has::<PlayerMarker>();

        let our_client = if we_are_player {
            let our_client = server.get_client(this_id)?;
            Some(our_client)
        } else {
            None
        };

        let our_dispatch = this.get::<&EntityDispatch>().unwrap();
        self.for_all_entities_nearby(ecs, world, dim, loc, move |other| {

            if other.entity() == this.entity() {
                return Ok(());
            }

            let other_id = *other.get::<&NetworkID>().unwrap();

            if other.has::<PlayerMarker>() {
                
                let other_client = server.get_client(other_id)?;
                our_dispatch.send_to_player(this_id, this, other_client)?;
            }

            if we_are_player {
                other.get::<&EntityDispatch>().unwrap().send_to_player(other_id, other, our_client.unwrap())?;
            }

            Ok(())

        })?;

        server.get_client(this_id)?.unload_entities(&us_to_unload)?;
        Ok(())
    }

    pub fn unload_entities_for(&self, ecs: &World, server: &Server, world: &GameWorld, player: EntityRef, dim: Location, loc: impl Iterator<Item = ChunkPosition>) -> anyhow::Result<()> {
        let mut us_to_unload = vec![];

        let this_id = *player.get::<&NetworkID>().unwrap();
        self.for_all_entities_nearby(ecs, world, dim, loc, |other| {

            if other.entity() == player.entity() {
                return Ok(());
            }

            us_to_unload.push(*other.get::<&NetworkID>().unwrap());

            if other.has::<PlayerMarker>() {
                let other_id = *other.get::<&NetworkID>().unwrap();
                let other_client = server.get_client(other_id)?;
                other_client.unload_entities(&[this_id])?;
            }

            Ok(())

        })?;

        let cl = server.get_client(this_id)?;
        if !cl.is_disconnected() {
            cl.unload_entities(&us_to_unload)?;
        }

        Ok(())
    }

    pub fn for_all_entities_nearby<F: FnMut(EntityRef) -> anyhow::Result<()>>(&self, ecs: &World, world: &GameWorld, dim: Location, locations: impl Iterator<Item = ChunkPosition>, mut f: F) -> anyhow::Result<()> {

        for v in locations {
            if let Some(loaded_chunk) = world.get_chunk(ChunkLocation {
                location: dim,
                position: v
            }) {
                for entity in loaded_chunk.2.iter() {
    
                    let other = ecs.entity(*entity)?;
    
                    (f)(other)?;
                }
            }
        }
        Ok(())
    }


    // /// TODO unify these. make it prettier.
    // pub fn unload_entities_in_chunks(&self, server: &Server, world: &GameWorld, this_entity: EntityRef, check_us: bool, values: impl Iterator<Item = ChunkPosition>) -> anyhow::Result<()> {

    //     let ecs = self.ecs().borrow();
    //     let loc = this_entity.get::<&EntityLocation>().unwrap().location;

    //     let we_are_player = this_entity.has::<PlayerMarker>();

    //     let our_id = *this_entity.get::<&NetworkID>().unwrap();

    //     for v in values {
    //         if let Some(loaded_chunk) = world.get_chunk(ChunkLocation {
    //             position: v,
    //             location: loc
    //         }) {
    //             for entity in loaded_chunk.2.iter() {
    //                 if *entity == this_entity.entity() {
    //                     continue;
    //                 }
    
    //                 let other = ecs.entity(*entity)?;
    //                 let other_id = *other.get::<&NetworkID>().unwrap();
    
    
    //                 if we_are_player && check_us {
    //                     // if we are a player
    //                     let client = server.get_client(our_id)?;
                        

    //                     if client.client_knows_entity(other_id) {
    //                         client.unload_entities(&[other_id])?;
    //                     }
    //                 } 
                    
    //                 if other.has::<PlayerMarker>() {
    //                     // if the other thing is a player
    //                     let client = server.get_client(*other.get::<&NetworkID>().unwrap())?;
                        

    //                     if client.client_knows_entity(our_id) {
    //                         client.unload_entities(&[our_id])?;
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     Ok(())
    // }

    // pub fn send_entities_in_chunks(&self, this_entity: EntityRef, values: impl Iterator<Item = ChunkPosition>) -> anyhow::Result<()> {

    //     let ecs = self.ecs().borrow();
    //     let loc = this_entity.get::<&EntityLocation>().unwrap().location;
    //     let world = self.resources.get::<GameWorld>();
    //     let server = self.resources.get::<Server>();

    //     let we_are_player = this_entity.has::<PlayerMarker>();

    //     let our_id = *this_entity.get::<&NetworkID>().unwrap();

    //     for v in values {
    //         if let Some(loaded_chunk) = world.get_chunk(ChunkLocation {
    //             position: v,
    //             location: loc
    //         }) {
    //             for entity in loaded_chunk.2.iter() {
    //                 if *entity == this_entity.entity() {
    //                     continue;
    //                 }
    
    //                 let other = ecs.entity(*entity)?;
    //                 let other_id = *other.get::<&NetworkID>().unwrap();
    
    
    //                 if we_are_player {
    //                     // if we are a player
    //                     let client = server.get_client(our_id)?;
                        

    //                     if !client.client_knows_entity(other_id) {
    //                         other.get::<&EntityDispatch>().unwrap().send_to_player(other, client)?;
    //                     }
    //                 } 
                    
    //                 if other.has::<PlayerMarker>() {
    //                     // if the other thing is a player
    //                     let client = server.get_client(*other.get::<&NetworkID>().unwrap())?;
                        

    //                     if !client.client_knows_entity(our_id) {
    //                         this_entity.get::<&EntityDispatch>().unwrap().send_to_player(this_entity, client)?;
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     Ok(())
    // }
}