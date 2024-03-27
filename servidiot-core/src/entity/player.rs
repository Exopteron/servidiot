use std::sync::Arc;

use servidiot_ecs::EntityRef;
use servidiot_network::server::{Client, id::NetworkID};
use servidiot_primitives::{metadata::{Metadata, MetadataItem}, position::EntityLocation};
use servidiot_yggdrasil::authenticate::Profile;

use super::Entity;

pub struct PlayerEntity;
pub struct PlayerMarker;

impl Entity for PlayerEntity {
    fn send_to_player(&self, this: EntityRef, cl: &Client) -> anyhow::Result<()> {
        
        let profile = this.get::<&Arc<Profile>>().unwrap().clone();
        tracing::info!("Sending {} to {}", profile.name, cl.profile.name);

        let id = *this.get::<&NetworkID>().unwrap();
        let pos = this.get::<&EntityLocation>().unwrap().position;

        let mut temp_meta = Metadata::default();
        temp_meta.insert(6, MetadataItem::Float(20.0));
        cl.send_player(id, &profile, pos, temp_meta)

    }
}