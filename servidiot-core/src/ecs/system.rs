use std::{any::type_name, marker::PhantomData, sync::Arc};

use super::{resources::Resources, Ecs};

pub type SysResult<T = ()> = anyhow::Result<T>;
pub type SystemFn<State> = Box<dyn FnMut(&mut State) -> SysResult>;

pub struct System<State> {
    func: SystemFn<State>,
    name: String,
}

impl<State> System<State> {
    pub fn from_fn<F: FnMut(&mut State) -> SysResult + 'static>(f: F) -> Self {
        Self {
            func: Box::new(f),
            name: type_name::<F>().to_string(),
        }
    }
}

pub struct SystemExecutor<State> {
    systems: Vec<System<State>>,
    index: usize
}
impl<State> Default for SystemExecutor<State> {
    fn default() -> Self {
        Self::new()
    }
}
impl<State> SystemExecutor<State> {

    /// Creates a new system executor.
    pub fn new() -> Self {
        Self {
            systems: vec![],
            index: 0
        }
    }

    pub fn add_system<F: FnMut(&mut State) -> SysResult + 'static>(&mut self, f: F) {
        self.systems.push(System::from_fn(f));
    }

    pub fn add_system_with_name<F: FnMut(&mut State) -> SysResult + 'static>(
        &mut self,
        name: String,
        f: F,
    ) {
        self.systems.push(System {
            name,
            func: Box::new(f),
        });
    }

    pub fn group<T: 'static>(&mut self) -> GroupBuilder<'_, State, T>
    where
        State: HasResources,
    {
        GroupBuilder {
            executor: self,
            p: PhantomData,
        }
    }

    pub fn run_systems(&mut self, state: &mut State) where State: HasEcs {
        state.ecs_mut().set_current_system_index(self.index);
        for s in &mut self.systems {
            if let Err(e) = (s.func)(state) {
                log::error!("System {:?} errored: {e:?}", s.name)
            }
        }
        self.index += 1;
    }
}

pub struct GroupBuilder<'a, State: HasResources, R> {
    executor: &'a mut SystemExecutor<State>,
    p: PhantomData<R>,
}
impl<'a, State: HasResources + 'static, R: 'static> GroupBuilder<'a, State, R> {
    pub fn add_system<F: FnMut(&mut State, &mut R) -> SysResult + 'static>(&mut self, f: F) -> &mut Self {
        self.executor
            .add_system_with_name(type_name::<F>().to_string(), Self::make_fn(f));
        self
    }

    fn make_fn(
        mut f: impl FnMut(&mut State, &mut R) -> SysResult,
    ) -> impl FnMut(&mut State) -> SysResult {
        move |state| {
            let res = state.resources();
            let mut r = res.get_mut::<R>()?;
            f(state, &mut r)
        }
    }
}

pub trait HasEcs {
    fn ecs(&self) -> &Ecs;
    fn ecs_mut(&mut self) -> &mut Ecs;
}

pub trait HasResources {
    fn resources(&self) -> Arc<Resources>;
}
