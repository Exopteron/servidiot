use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
};

/// An event.
pub trait Event: 'static {
    /// Whether or not this event is to be deferred or handled immediately.
    const IMMEDIATE: bool;
}

type EventTransformerFn<State> = Box<dyn Fn(&State, &mut dyn Any) -> anyhow::Result<bool>>;

type ImmediateEventHandler<State> = Box<dyn Fn(&State, &dyn Any) -> anyhow::Result<()>>;

pub struct EventManager<State> {
    immediate_handlers: HashMap<TypeId, Vec<ImmediateEventHandler<State>>>,
    transformers: HashMap<TypeId, Vec<EventTransformerFn<State>>>,
    deferred: RefCell<HashMap<TypeId, Vec<Box<dyn Any>>>>,
}
impl<State> Default for EventManager<State> {
    fn default() -> Self {
        Self::new()
    }
}
impl<State> EventManager<State> {
    pub fn new() -> Self {
        Self {
            immediate_handlers: Default::default(),
            transformers: Default::default(),
            deferred: RefCell::new(Default::default()),
        }
    }

    /// Registers an immediate event handler.
    pub fn register_handler<E: Event>(
        &mut self,
        f: impl Fn(&State, &E) -> anyhow::Result<()> + 'static,
    ) {
        self.immediate_handlers
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(move |state, val| {
                f(state, val.downcast_ref().expect("checked"))
            }))
    }

    /// Register a transformer. If the transformer callback returns `false`, cancel the event.
    pub fn register_transformer<E: Event>(
        &mut self,
        f: impl Fn(&State, &mut E) -> anyhow::Result<bool> + 'static,
    ) {
        self.transformers
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(move |state, val| {
                f(state, val.downcast_mut().expect("checked"))
            }))
    }

    /// Post an event to the event handler.
    pub fn post_event<E: Event>(&self, state: &State, mut event: E) -> anyhow::Result<()> {
        if let Some(transformers) = self.transformers.get(&TypeId::of::<E>()) {
            for t in transformers {
                if !t(state, &mut event)? {
                    // event cancelled
                    return Ok(());
                }
            }
        }

        if E::IMMEDIATE {
            if let Some(h) = self.immediate_handlers.get(&TypeId::of::<E>()) {
                for handler in h {
                    handler(state, &event)?;
                }
            }
        } else {
            self.deferred
                .borrow_mut()
                .entry(TypeId::of::<E>())
                .or_default()
                .push(Box::new(event));
        }
        Ok(())
    }

    /// Gather the deferred events to handle now.
    pub fn deferred_events<E: Event>(&self) -> impl Iterator<Item = E> {
        let v = self
            .deferred
            .borrow_mut()
            .remove(&TypeId::of::<E>())
            .unwrap_or_default();
        v.into_iter().map(|v| *v.downcast().unwrap())
    }
}
