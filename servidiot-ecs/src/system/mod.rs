

type SystemFn<State> =
    dyn Fn(&State) -> anyhow::Result<()>;

pub struct SystemExecutor<State> {
    systems: Vec<Box<SystemFn<State>>>,
}
impl<State> Default for SystemExecutor<State> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> SystemExecutor<State> {
    pub fn new() -> Self {
        Self { systems: vec![] }
    }

    pub fn add_system(
        &mut self,
        s: impl Fn(&State) -> anyhow::Result<()> + 'static,
    ) -> &mut Self {
        self.systems.push(Box::new(s));
        self
    }

    pub fn run_systems(&self, state: &State)  {
        for sys in &self.systems {
            if let Err(e) = sys(state) {
                tracing::error!("System error: {:?}", e);
            }
        }
    }
}
