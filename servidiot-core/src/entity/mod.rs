use servidiot_ecs::EntityRef;
use servidiot_network::server::{id::NetworkID, Client};

pub mod player;

pub trait Entity {
    fn send_to_player(&self, this: EntityRef, cl: &Client) -> anyhow::Result<()>;
}

/// Dynamic dispatch of certain functions on an entity.
pub struct EntityDispatch(Box<dyn Entity + Send + Sync>);

impl EntityDispatch {
    pub fn new(e: impl Entity + Send + Sync + 'static) -> Self {
        Self(Box::new(e))
    }

    pub fn send_to_player(&self, this_id: NetworkID, this: EntityRef, cl: &Client) -> anyhow::Result<()> {
        if !cl.client_knows_entity(this_id) {
            self.0.send_to_player(this, cl)?;
        }
        Ok(())
    }
}