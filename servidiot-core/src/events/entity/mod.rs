use servidiot_ecs::Entity;
use servidiot_primitives::position::Position;
use servidiot_utils::events::Event;

use crate::world::view::View;

pub struct EntityMoveEvent {
    pub entity: Entity,
    pub old_pos: Position,
    pub new_pos: Position
}
impl Event for EntityMoveEvent {
    const IMMEDIATE: bool = false;
}

pub struct PlayerViewChangeEvent {
    pub entity: Entity,
    pub old_view: View,
    pub new_view: View
}
impl Event for PlayerViewChangeEvent {
    const IMMEDIATE: bool = false;
}


