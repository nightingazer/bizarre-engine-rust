pub enum ScheduleType {
    Frame,
    Tick,
}

pub struct Schedule {
    // pub tick_dispatcher: specs::Dispatcher<'static, 'static>,
    pub frame_dispatcher: specs::Dispatcher<'static, 'static>,
}

impl Default for Schedule {
    fn default() -> Self {
        Self {
            frame_dispatcher: specs::DispatcherBuilder::new().build(),
        }
    }
}

pub struct ScheduleBuilder {
    frame_dispatcher: Option<specs::DispatcherBuilder<'static, 'static>>,
}

impl Default for ScheduleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleBuilder {
    pub fn new() -> Self {
        Self {
            frame_dispatcher: Some(specs::DispatcherBuilder::new()),
        }
    }

    pub fn with_frame_system<F>(
        &mut self,
        system: F,
        name: &str,
        dependencies: &[&str],
    ) -> &mut Self
    where
        F: for<'a> specs::System<'a> + 'static + Send,
    {
        self.frame_dispatcher
            .as_mut()
            .unwrap()
            .add(system, name, dependencies);
        self
    }

    pub fn build(&mut self) -> Schedule {
        let frame_dispatcher = self.frame_dispatcher.take().unwrap();
        Schedule {
            frame_dispatcher: frame_dispatcher.build(),
        }
    }
}
