use std::{cell::RefCell, sync::Arc, rc::Rc};

use ecs::{Ecs, system::{SystemExecutor, HasEcs, HasResources}, resources::Resources};
pub use servidiot_network::server::Server;


pub mod ecs;
pub(crate) mod world;

pub struct MinecraftServer {
    /// The ECS.
    ecs: Ecs,
    /// System executor.
    systems: Rc<RefCell<SystemExecutor<Self>>>,
    /// Resources.
    resources: Arc<Resources>
}

impl MinecraftServer {
    /// Binds the server to an address.
    pub fn new(server: Server) -> anyhow::Result<Self> {
        let mut resources = Resources::default();
        resources.insert(server);
        let ecs = Ecs::new();
        let mut systems = SystemExecutor::new();
        ecs::systems::register_systems(&mut systems, &mut resources);

        Ok(Self {
            resources: Arc::new(resources),
            ecs,
            systems: Rc::new(RefCell::new(systems))
        })
        
    }

    /// Update the game.
    pub fn update(&mut self) {
        let s = self.systems.clone();
        let mut systems = s.borrow_mut();
        systems.run_systems(self);
    }
}

impl HasEcs for MinecraftServer {
    fn ecs(&self) -> &Ecs {
        &self.ecs
    }

    fn ecs_mut(&mut self) -> &mut Ecs {
        &mut self.ecs
    }
}
impl HasResources for MinecraftServer {
    fn resources(&self) -> Arc<Resources> {
        Arc::clone(&self.resources)
    }
}