use hecs::{World, Component, NoSuchEntity, Entity, DynamicBundle, EntityRef, Ref, ComponentError, RefMut, QueryBorrow, QueryOne, Query};

use self::event::EventTracker;

pub mod event;
pub mod resources;
pub mod system;
pub mod systems;
pub mod entities;

pub struct Ecs {
    world: World,
    event_tracker: EventTracker,
}

impl Default for Ecs {
    fn default() -> Self {
        Self::new()
    }
}
impl Ecs {
    /// Creates a new ECS.
    pub fn new() -> Self {
        Self {
            world: World::new(),
            event_tracker: EventTracker::default()
        }
    }
    /// Inserts an event into the world.
    pub fn insert_event<T: Component>(&mut self, event: T) {
        let entity = self.world.spawn((event,));
        self.event_tracker.insert_event(entity);
    }

    /// Sets the index of the currently executing system,
    /// used for event tracking.
    pub fn set_current_system_index(&mut self, index: usize) {
        self.event_tracker.set_current_system_index(index);
    }

        /// Adds a component to an entity.
    ///
    /// Do not use this function to add events. Use [`Ecs::insert_event`]
    /// instead.
    pub fn insert(
        &mut self,
        entity: Entity,
        component: impl Component,
    ) -> Result<(), NoSuchEntity> {
        self.world.insert_one(entity, component)
    }
    /// Defers removing an entity until before the next time this system
    /// runs, allowing it to be observed by systems one last time.
    pub fn defer_despawn(&mut self, entity: Entity) {
        // a bit of a hack - but this will change once
        // hecs allows taking out components of a despawned entity
        self.event_tracker.insert_event(entity);
    }
    /// Removes a component from an entity and returns it.
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        self.world.remove_one(entity)
    }

    /// Adds an event component to an entity and schedules
    /// it to be removed immediately before the current system
    /// runs again. Thus, all systems have exactly one chance
    /// to observe the event before it is dropped.
    pub fn insert_entity_event<T: Component>(
        &mut self,
        entity: Entity,
        event: T,
    ) -> Result<(), NoSuchEntity> {
        self.insert(entity, event)?;
        self.event_tracker.insert_entity_event::<T>(entity);
        Ok(())
    }
    /// Should be called before each system runs.
    pub fn remove_old_events(&mut self) {
        self.event_tracker.remove_old_events(&mut self.world);
    }

    pub fn spawn(&mut self, c: impl DynamicBundle) -> Entity {
        self.world.spawn(c)
    }
    /// Returns an `EntityRef` for an entity.
    pub fn entity(&self, entity: Entity) -> Result<EntityRef, NoSuchEntity> {
        self.world.entity(entity)
    }

    /// Gets a component of an entity.
    pub fn get<T: Component>(&self, entity: Entity) -> Result<Ref<T>, ComponentError> {
        self.world.get::<&T>(entity)
    }

    /// Mutably gets a component of an entity.
    pub fn get_mut<T: Component>(&self, entity: Entity) -> Result<RefMut<T>, ComponentError> {
        self.world.get::<&mut T>(entity)
    }

    /// Returns an iterator over all entities that match a query parameter.
    pub fn query<Q: Query>(&self) -> QueryBorrow<Q> {
        self.world.query()
    }

    pub fn query_one<Q: Query>(&self, entity: Entity) -> Result<QueryOne<'_, Q>, NoSuchEntity> {
        self.world.query_one(entity)
    }
}
