use specs::{Dispatcher, DispatcherBuilder, System};

pub enum ScheduleType {
    Frame,
    Setup,
}

pub struct Schedule {
    pub frame_dispatcher: Dispatcher<'static, 'static>,
    pub setup_dispatcher: Dispatcher<'static, 'static>,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            frame_dispatcher: DispatcherBuilder::new().build(),
            setup_dispatcher: DispatcherBuilder::new().build(),
        }
    }
}

pub struct ScheduleBuilder {
    // TODO: expose barriers to app_builder
    pub(crate) frame_dispatcher: Option<DispatcherBuilder<'static, 'static>>,
    pub(crate) setup_dispatcher: Option<DispatcherBuilder<'static, 'static>>,
}

impl Default for ScheduleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleBuilder {
    pub fn new() -> Self {
        Self {
            frame_dispatcher: Some(DispatcherBuilder::new()),
            setup_dispatcher: Some(DispatcherBuilder::new()),
        }
    }

    #[deprecated]
    pub fn with_frame_system<F>(
        &mut self,
        system: F,
        name: &str,
        dependencies: &[&str],
    ) -> &mut Self
    where
        F: for<'a> System<'a> + 'static + Send,
    {
        self.frame_dispatcher
            .as_mut()
            .unwrap()
            .add(system, name, dependencies);
        self
    }

    pub fn with_system<F>(
        &mut self,
        schedule_type: ScheduleType,
        system: F,
        name: &str,
        dependencies: &[&str],
    ) -> &mut Self
    where
        F: for<'a> System<'a> + 'static + Send,
    {
        let dispatcher = match schedule_type {
            ScheduleType::Frame => self.frame_dispatcher.as_mut().unwrap(),
            ScheduleType::Setup => self.setup_dispatcher.as_mut().unwrap(),
        };
        dispatcher.add(system, name, dependencies);
        self
    }

    pub fn build(&mut self) -> Schedule {
        let frame_dispatcher = self.frame_dispatcher.take().unwrap();
        let init_dispatcher = self.setup_dispatcher.take().unwrap();
        Schedule {
            frame_dispatcher: frame_dispatcher.build(),
            setup_dispatcher: init_dispatcher.build(),
        }
    }
}
