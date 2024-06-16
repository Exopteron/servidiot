use std::{
    collections::{hash_map::Entry, HashSet},
    hash::Hash,
};

use fxhash::{FxHashMap, FxHashSet};
use servidiot_primitives::{
    chunk::Chunk,
    position::{ChunkLocation, ChunkPosition, DimensionID, Location},
};
use slotmap::{new_key_type, SlotMap};
use view::View;

pub mod view;
mod world;

struct TrackedEntity<EntityData> {
    value: EntityData,
    inhabits: ChunkLocation,
    load_radius: Option<u32>,
    waiting_on: FxHashSet<ChunkLocation>,
    known_entities: FxHashSet<TrackedEntityKey>,
}

new_key_type! { pub struct TrackedEntityKey; }

struct ChunkData {
    entities_within: FxHashSet<TrackedEntityKey>,
    ticket_count: u32,
}
type DimensionData = FxHashMap<ChunkPosition, ChunkData>;

pub struct TrackedWorld<EntityData> {
    chunk_data: FxHashMap<u32, FxHashMap<DimensionID, DimensionData>>,
    entity_store: SlotMap<TrackedEntityKey, TrackedEntity<EntityData>>,
    entities_awaiting_chunks: FxHashMap<ChunkLocation, FxHashSet<TrackedEntityKey>>,
    event_queue: Vec<TrackedWorldEvent<EntityData>>,
}

impl<EntityData> Default for TrackedWorld<EntityData> {
    fn default() -> Self {
        Self {
            chunk_data: Default::default(),
            entity_store: Default::default(),
            entities_awaiting_chunks: Default::default(),
            event_queue: Default::default(),
        }
    }
}

const ENTITY_LOAD_DISTANCE: u32 = 8;

impl<EntityData> TrackedWorld<EntityData> {
    pub fn new() -> Self {
        Self::default()
    }

    fn dimension(&mut self, loc: Location) -> &mut DimensionData {
        self.chunk_data
            .entry(loc.world)
            .or_default()
            .entry(loc.dimension)
            .or_default()
    }

    pub fn poll_events(&mut self) -> impl Iterator<Item = TrackedWorldEvent<EntityData>> + '_ {
        self.event_queue.drain(..)
    }

    fn try_chunk(&mut self, loc: ChunkLocation) -> Option<&mut ChunkData> {
        self.dimension(loc.location).get_mut(&loc.position)
    }

    fn chunk(&mut self, loc: ChunkLocation) -> &mut ChunkData {
        self.try_chunk(loc).unwrap()
    }

    pub fn entity(&self, i: TrackedEntityKey) -> &EntityData {
        &self.entity_store[i].value
    }

    /// Adds an entity to the tracker. Returns `None` if this entity does not load chunks
    /// but was added to an unloaded chunk.
    pub fn add_entity(
        &mut self,
        d: EntityData,
        pos: ChunkLocation,
        load_radius: Option<u32>,
    ) -> Option<TrackedEntityKey> {
        if load_radius.is_none() && self.try_chunk(pos).is_none() {
            return None;
        }
        let inserted = self.entity_store.insert(TrackedEntity {
            value: d,
            inhabits: pos,
            load_radius,
            waiting_on: Default::default(),
            known_entities: Default::default(),
        });

        self.add_entity_to_chunk(inserted, pos, load_radius.is_some());

        if let Some(load_radius) = load_radius {
            self.add_chunks_to_entity_view(
                inserted,
                View::new(pos, load_radius).chunks().into_iter(),
            );
        }

        self.handle_entity_visibilities(inserted, None, View::new(pos, ENTITY_LOAD_DISTANCE));

        Some(inserted)
    }

    fn event(&mut self, e: TrackedWorldEvent<EntityData>) {
        self.event_queue.push(e);
    }

    fn add_entity_to_chunk(&mut self, e: TrackedEntityKey, c: ChunkLocation, wait: bool) {
        if let Some(t) = self.try_chunk(c) {
            t.entities_within.insert(e);
        } else if wait {
            self.wait_on_chunk(e, c);
        } else {
            panic!("Attempted to add to unloaded chunk");
        }
    }

    fn wait_on_chunk(&mut self, e: TrackedEntityKey, c: ChunkLocation) {
        self.entity_store[e].waiting_on.insert(c);
        let en = self.entities_awaiting_chunks.entry(c);

        let request_load = matches!(en, Entry::Vacant(_));
        en.or_default().insert(e);
        if request_load {
            self.event(TrackedWorldEvent::RequestLoad(c));
        }
    }

    fn unload_entity(
        &mut self,
        us: TrackedEntityKey,
        event: bool,
    ) -> Option<TrackedEntity<EntityData>> {
        if let Some(data) = self.entity_store.remove(us) {
            for value in View::new(data.inhabits, ENTITY_LOAD_DISTANCE).chunks() {
                if let Some(ch) = self.try_chunk(value) {
                    for other in ch.entities_within.clone() {
                        let other_entity = &mut self.entity_store[other];
                        if other != us && other_entity.load_radius.is_some() {
                            other_entity.known_entities.remove(&us);
                            self.event(TrackedWorldEvent::EntityNoLongerViewsEntities(
                                other,
                                vec![us],
                            ));
                        }
                    }
                }
            }

            for v in &data.waiting_on {
                if let Some(value) = self.entities_awaiting_chunks.get_mut(v) {
                    value.remove(&us);
                }
            }

            if event {
                self.event(TrackedWorldEvent::UnloadEntity(data.value));
                None
            } else {
                Some(data)
            }
        } else {
            panic!("Entity {us:?} not present")
        }
    }

    fn handle_entity_visibilities(&mut self, e: TrackedEntityKey, old: Option<View>, new: View) {
        let is_loader = if let Some(v) = self.entity_store.get(e) {
            v.load_radius.is_some()
        } else {
            false
        };

        if is_loader {
            let mut new_entities = FxHashSet::default();
            for chunk in new.chunks() {
                if let Some(chunk) = self.try_chunk(chunk) {
                    for other_entity in chunk.entities_within.clone() {
                        if other_entity != e
                            && self.entity_store[e].known_entities.insert(other_entity)
                        {
                            new_entities.insert(other_entity);
                            self.event(TrackedWorldEvent::EntityViewsEntities(
                                e,
                                vec![other_entity],
                            ))
                        }
                    }
                }
            }

            let our_pos = self.entity_store[e].inhabits;
            for value in self.entity_store[e].known_entities.clone() {
                if !new_entities.contains(&value) {
                    if !self.entity_store.contains_key(value) {
                        self.entity_store[e].known_entities.remove(&value);
                        self.event(TrackedWorldEvent::EntityNoLongerViewsEntities(
                            e,
                            vec![value],
                        ));
                        continue;
                    }

                    let dist = our_pos
                        .position
                        .distance_squared(&self.entity_store[value].inhabits.position);
                    if dist > (ENTITY_LOAD_DISTANCE as i32 * ENTITY_LOAD_DISTANCE as i32) {
                        self.entity_store[e].known_entities.remove(&value);
                        self.event(TrackedWorldEvent::EntityNoLongerViewsEntities(
                            e,
                            vec![value],
                        ))
                    }
                }
            }
        }

        let chunks_to_notify_new = if let Some(ref old) = old {
            new.difference(old).collect::<Vec<_>>()
        } else {
            new.chunks().into_iter().collect()
        };

        for chunk in chunks_to_notify_new {
            if let Some(chunk) = self.try_chunk(chunk) {
                for other_entity in chunk.entities_within.clone() {
                    let other_entity_data = &mut self.entity_store[other_entity];
                    if other_entity_data.load_radius.is_some()
                        && other_entity_data.known_entities.insert(e)
                    {
                        self.event(TrackedWorldEvent::EntityViewsEntities(
                            other_entity,
                            vec![e],
                        ))
                    }
                }
            }
        }

        if let Some(ref old) = old {
            for chunk in old.difference(&new) {
                if let Some(chunk) = self.try_chunk(chunk) {
                    for other_entity in chunk.entities_within.clone() {
                        let other_entity_data = &mut self.entity_store[other_entity];
                        if other_entity_data.load_radius.is_some()
                            && other_entity_data.known_entities.remove(&e)
                        {
                            self.event(TrackedWorldEvent::EntityNoLongerViewsEntities(
                                other_entity,
                                vec![e],
                            ))
                        }
                    }
                }
            }
        }
    }

    fn unload_chunk(&mut self, pos: ChunkLocation) {
        if let Some(chunk) = self.dimension(pos.location).remove(&pos.position) {
            let mut entities = vec![];
            for entity in chunk.entities_within {
                let entity = self.unload_entity(entity, false).unwrap();
                entities.push(entity.value);
            }

            self.event(TrackedWorldEvent::UnloadChunk(pos, entities))
        }
    }

    fn add_chunks_to_entity_view(
        &mut self,
        e: TrackedEntityKey,
        i: impl Iterator<Item = ChunkLocation>,
    ) {
        let mut views = vec![];
        for chunk in i {
            if let Some(data) = self.dimension(chunk.location).get_mut(&chunk.position) {
                data.ticket_count += 1;
                views.push(chunk);
            } else {
                self.wait_on_chunk(e, chunk);
            }
        }
        if !views.is_empty() {
            self.event(TrackedWorldEvent::EntityViewsChunks(e, views));
        }
    }

    pub fn move_entity(&mut self, e: TrackedEntityKey, new_pos: ChunkLocation) {
        let load_radius = self.entity_store[e].load_radius;

        let old_pos = self.entity_store[e].inhabits;

        self.chunk(old_pos).entities_within.remove(&e);

        self.entity_store[e].inhabits = new_pos;

        if let Some(load_radius) = load_radius {
            self.add_entity_to_chunk(e, new_pos, true);

            let old_view = View::new(old_pos, load_radius);
            let new_view = View::new(new_pos, load_radius);

            let new_view_chunks = new_view.chunks();
            let new_chunks = new_view.difference(&old_view);
            let old_chunks = old_view.difference(&new_view);

            // remove our ticket from chunks no longer in our view
            let mut no_longer = vec![];
            for chunk in old_chunks {
                let should_remove = {
                    let ch = self.chunk(chunk);
                    ch.ticket_count = ch.ticket_count.saturating_sub(1);
                    ch.ticket_count == 0
                };
                no_longer.push(chunk);

                if should_remove {
                    self.unload_chunk(chunk);
                }
            }
            if !no_longer.is_empty() {
                self.event(TrackedWorldEvent::EntityNoLongerViewsChunks(e, no_longer));
            }

            // stop waiting on chunks no longer in our view
            self.entity_store[e].waiting_on.retain(|v| {
                if !new_view_chunks.contains(v) {
                    if let Some(awaiting) = self.entities_awaiting_chunks.get_mut(v) {
                        awaiting.remove(&e);
                    }
                    false
                } else {
                    true
                }
            });

            self.add_chunks_to_entity_view(e, new_chunks);
        } else if let Some(c) = self.try_chunk(new_pos) {
            c.entities_within.insert(e);
        } else {
            self.unload_entity(e, true);
        }

        self.handle_entity_visibilities(
            e,
            Some(View::new(old_pos, ENTITY_LOAD_DISTANCE)),
            View::new(new_pos, ENTITY_LOAD_DISTANCE),
        );
    }

    pub fn add_chunk(&mut self, chunk: ChunkLocation) {
        let mut ticket_count = 0;
        let mut entities_within = FxHashSet::default();
        if let Some(awaiting) = self.entities_awaiting_chunks.remove(&chunk) {
            for entity in awaiting {
                ticket_count += 1;

                let en = &self.entity_store[entity];
                if en.inhabits == chunk {
                    entities_within.insert(entity);
                }
                if en.load_radius.is_some() {
                    self.event(TrackedWorldEvent::EntityViewsChunks(entity, vec![chunk]));
                }
            }
        }

        if ticket_count == 0 {
            tracing::error!("No one was waiting on chunk {:?}", chunk);
            return; // don't add it - no one wants it
        }
        self.dimension(chunk.location).insert(
            chunk.position,
            ChunkData {
                entities_within,
                ticket_count: 0,
            },
        );
    }
}

#[derive(Debug, PartialEq)]
pub enum TrackedWorldEvent<EntityData> {
    UnloadChunk(ChunkLocation, Vec<EntityData>),
    UnloadEntity(EntityData),
    RequestLoad(ChunkLocation),
    EntityViewsChunks(TrackedEntityKey, Vec<ChunkLocation>),
    EntityNoLongerViewsChunks(TrackedEntityKey, Vec<ChunkLocation>),
    EntityViewsEntities(TrackedEntityKey, Vec<TrackedEntityKey>),
    EntityNoLongerViewsEntities(TrackedEntityKey, Vec<TrackedEntityKey>),
}

#[cfg(test)]
mod tests {
    use servidiot_primitives::{
        chunk::Chunk,
        position::{ChunkLocation, ChunkPosition, Location},
    };

    use crate::{TrackedWorld, TrackedWorldEvent};

    macro_rules! ensure_has_event {
        ($events:expr, PAT $event:pat) => {
            'blk: {
                for event in $events {
                    if matches!(event, $event) {
                        break 'blk;
                    }
                }
                panic!("Did not find matching event")
            }
        };

        ($events:expr, EXPR $event:expr) => {
            'blk: {
                for event in $events {
                    if event.eq(&$event) {
                        break 'blk;
                    }
                }
                panic!("Did not find matching event")
            }
        };
    }

    fn loc(x: i32, z: i32) -> ChunkLocation {
        ChunkLocation::new(ChunkPosition::new(x, z), Location::new(0, 0))
    }

    #[test]
    fn spawn_one_entity() {
        let mut tracker = TrackedWorld::<u64>::new();
        let _ = tracker
            .add_entity(
                0,
                loc(0, 0),
                Some(2),
            )
            .unwrap();

        ensure_has_event!(
            tracker.poll_events(),
            PAT TrackedWorldEvent::RequestLoad(ChunkLocation {
                location: _,
                position: ChunkPosition { x: 0, z: 0 }
            })
        )
    }


    #[test]
    fn player_leaves_npc_view() {
        let mut tracker = TrackedWorld::<u64>::new();
        let player = tracker
            .add_entity(
                0,
                loc(0, 0),
                Some(2),
            )
            .unwrap();

        let events = tracker.poll_events().collect::<Vec<_>>();
        ensure_has_event!(
            &events,
            PAT TrackedWorldEvent::RequestLoad(ChunkLocation {
                location: _,
                position: ChunkPosition { x: 0, z: 0 }
            })
        );

        for event in events {
            if let TrackedWorldEvent::RequestLoad(c) = event {
                tracker.add_chunk(c);
            }
        }

        let npc = tracker
            .add_entity(
                1,
                loc(0, 0),
                None,
            )
            .unwrap();

            ensure_has_event!(
                tracker.poll_events(),
                EXPR TrackedWorldEvent::EntityViewsEntities(player, vec![npc])
            );

        tracker.move_entity(player, loc(10, 10));

        let polled = tracker.poll_events().collect::<Vec<_>>();
        ensure_has_event!(
            &polled,
            EXPR TrackedWorldEvent::EntityNoLongerViewsEntities(player, vec![npc])
        );
        ensure_has_event!(
            &polled,
            EXPR TrackedWorldEvent::EntityNoLongerViewsEntities(player, vec![npc])
        );

    }

    #[test]
    fn npc_leaves_player_view() {
        let mut tracker = TrackedWorld::<u64>::new();
        let player = tracker
            .add_entity(
                0,
                loc(0, 0),
                Some(2),
            )
            .unwrap();

        let events = tracker.poll_events().collect::<Vec<_>>();
        ensure_has_event!(
            &events,
            PAT TrackedWorldEvent::RequestLoad(ChunkLocation {
                location: _,
                position: ChunkPosition { x: 0, z: 0 }
            })
        );

        for event in events {
            if let TrackedWorldEvent::RequestLoad(c) = event {
                tracker.add_chunk(c);
            }
        }

        let npc = tracker
            .add_entity(
                1,
                loc(0, 0),
                None,
            )
            .unwrap();

            ensure_has_event!(
                tracker.poll_events(),
                EXPR TrackedWorldEvent::EntityViewsEntities(player, vec![npc])
            );

        tracker.move_entity(npc, loc(10, 10));

        let polled = tracker.poll_events().collect::<Vec<_>>();
        ensure_has_event!(
            &polled,
            EXPR TrackedWorldEvent::EntityNoLongerViewsEntities(player, vec![npc])
        );
        ensure_has_event!(
            &polled,
            EXPR TrackedWorldEvent::EntityNoLongerViewsEntities(player, vec![npc])
        );

    }

    // #[test]
    // fn world_track_test() {
    //     let mut tracker = TrackedWorld::<u64>::new();
    //     let us = tracker
    //         .add_entity(
    //             0,
    //             ChunkLocation::new(ChunkPosition::new(0, 0), Location::new(0, 0)),
    //             Some(2),
    //         )
    //         .unwrap();

    //     loop {
    //         let events = tracker.poll_events().collect::<Vec<_>>();
    //         if events.is_empty() {
    //             break;
    //         }
    //         for value in events {
    //             match value {
    //                 crate::TrackedWorldEvent::UnloadChunk(loc, _, en) => {
    //                     println!("Unloading {} with entities: {:?}", loc.position, en)
    //                 }
    //                 crate::TrackedWorldEvent::RequestLoad(loc) => {
    //                     println!("Loading {}", loc.position);
    //                     tracker.add_chunk(loc, Box::new(Chunk::new(loc.position)));
    //                 }
    //                 crate::TrackedWorldEvent::EntityViewsChunks(e, loc) => {
    //                     println!("Entity {e:?} now views {:?}", loc)
    //                 }
    //                 crate::TrackedWorldEvent::EntityNoLongerViewsChunks(e, loc) => {
    //                     println!("Entity {e:?} no longer views {:?}", loc)
    //                 }
    //                 crate::TrackedWorldEvent::EntityViewsEntities(e, v) => {
    //                     println!("Entity {e:?} now views entities: {v:?}")
    //                 }
    //                 crate::TrackedWorldEvent::EntityNoLongerViewsEntities(e, v) => {
    //                     println!("Entity {e:?} no longer views entities: {v:?}")
    //                 }
    //                 crate::TrackedWorldEvent::UnloadEntity(e) => println!("Unloading entity {e:?}"),
    //             }
    //         }
    //     }

    //     println!("\nMOVING\n");

    //     let npc = tracker
    //         .add_entity(
    //             1,
    //             ChunkLocation::new(ChunkPosition::new(1, 0), Location::new(0, 0)),
    //             None,
    //         )
    //         .unwrap();
    //     tracker.move_entity(
    //         us,
    //         ChunkLocation::new(ChunkPosition::new(1, 0), Location::new(0, 0)),
    //     );

    //     loop {
    //         let events = tracker.poll_events().collect::<Vec<_>>();
    //         if events.is_empty() {
    //             break;
    //         }
    //         for value in events {
    //             match value {
    //                 crate::TrackedWorldEvent::UnloadChunk(loc, _, en) => {
    //                     println!("Unloading {} with entities: {:?}", loc.position, en)
    //                 }
    //                 crate::TrackedWorldEvent::RequestLoad(loc) => {
    //                     println!("Loading {}", loc.position);
    //                     tracker.add_chunk(loc, Box::new(Chunk::new(loc.position)));
    //                 }
    //                 crate::TrackedWorldEvent::EntityViewsChunks(e, loc) => {
    //                     println!("Entity {e:?} now views {:?}", loc)
    //                 }
    //                 crate::TrackedWorldEvent::EntityNoLongerViewsChunks(e, loc) => {
    //                     println!("Entity {e:?} no longer views {:?}", loc)
    //                 }
    //                 crate::TrackedWorldEvent::EntityViewsEntities(e, v) => {
    //                     println!("Entity {e:?} now views entities: {v:?}")
    //                 }
    //                 crate::TrackedWorldEvent::EntityNoLongerViewsEntities(e, v) => {
    //                     println!("Entity {e:?} no longer views entities: {v:?}")
    //                 }
    //                 crate::TrackedWorldEvent::UnloadEntity(e) => println!("Unloading entity {e:?}"),
    //             }
    //         }
    //     }

    //     println!("\nMOVING AGAIN\n");

    //     tracker.move_entity(
    //         us,
    //         ChunkLocation::new(ChunkPosition::new(6, 0), Location::new(0, 0)),
    //     );

    //     loop {
    //         let events = tracker.poll_events().collect::<Vec<_>>();
    //         if events.is_empty() {
    //             break;
    //         }
    //         for value in events {
    //             match value {
    //                 crate::TrackedWorldEvent::UnloadChunk(loc, _, en) => {
    //                     println!("Unloading {} with entities: {:?}", loc.position, en)
    //                 }
    //                 crate::TrackedWorldEvent::RequestLoad(loc) => {
    //                     println!("Loading {}", loc.position);
    //                     tracker.add_chunk(loc, Box::new(Chunk::new(loc.position)));
    //                 }
    //                 crate::TrackedWorldEvent::EntityViewsChunks(e, loc) => {
    //                     println!("Entity {e:?} now views {:?}", loc)
    //                 }
    //                 crate::TrackedWorldEvent::EntityNoLongerViewsChunks(e, loc) => {
    //                     println!("Entity {e:?} no longer views {:?}", loc)
    //                 }
    //                 crate::TrackedWorldEvent::EntityViewsEntities(e, v) => {
    //                     println!("Entity {e:?} now views entities: {v:?}")
    //                 }
    //                 crate::TrackedWorldEvent::EntityNoLongerViewsEntities(e, v) => {
    //                     println!("Entity {e:?} no longer views entities: {v:?}")
    //                 }
    //                 crate::TrackedWorldEvent::UnloadEntity(e) => println!("Unloading entity {e:?}"),
    //             }
    //         }
    //     }
    // }
}
