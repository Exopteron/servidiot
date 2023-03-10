use hecs::{World, Entity, Component};

/// Function to remove an event from the ECS.
type EventRemoveFn = fn(&mut World, Entity);

fn entity_event_remove_fn<T: Component>() -> EventRemoveFn {
    |ecs, entity| {
        let _ = ecs.remove_one::<T>(entity);
    }
}

fn event_remove_fn(world: &mut World, event_entity: Entity) {
    let _ = world.despawn(event_entity);
}
#[derive(Default)]
pub struct EventTracker {
    events: Vec<Vec<(Entity, EventRemoveFn)>>,

    current_system_index: usize,
}
impl EventTracker {


    /// Adds an entity event to be tracked.
    pub fn insert_entity_event<T: Component>(&mut self, entity: Entity) {
        let events_vec = self.current_events_vec();
        events_vec.push((entity, entity_event_remove_fn::<T>()))
    }

    /// Adds an event to be tracked.
    pub fn insert_event(&mut self, event_entity: Entity) {
        let events_vec = self.current_events_vec();
        events_vec.push((event_entity, event_remove_fn));
    }

    /// Adds a custom function to run
    /// before the current systems executes again.
    #[allow(unused)]
    pub fn insert_custom(&mut self, entity: Entity, callback: fn(&mut World, Entity)) {
        let events_vec = self.current_events_vec();
        events_vec.push((entity, callback));
    }

    pub fn set_current_system_index(&mut self, index: usize) {
        self.current_system_index = index;
    }

    /// Deletes events that were triggered on the previous tick
    /// by the current system.
    pub fn remove_old_events(&mut self, world: &mut World) {
        let events_vec = self.current_events_vec();
        for (entity, remove_fn) in events_vec.drain(..) {
            remove_fn(world, entity);
        }
    }

    fn current_events_vec(&mut self) -> &mut Vec<(Entity, EventRemoveFn)> {
        while self.events.len() <= self.current_system_index {
            self.events.push(Vec::new());
        }
        &mut self.events[self.current_system_index]
    }
}
