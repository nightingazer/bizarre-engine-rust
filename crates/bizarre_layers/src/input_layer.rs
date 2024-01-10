use anyhow::Result;
use bizarre_core::{input::InputHandler, layer::Layer, schedule::ScheduleBuilder};
use bizarre_events::observer::EventBus;
use specs::WorldExt;

pub struct InputLayer {}

impl InputLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for InputLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl Layer for InputLayer {
    fn on_attach(
        &mut self,
        _: &EventBus,
        world: &mut specs::World,
        _schedule_builder: &mut ScheduleBuilder,
    ) -> Result<()> {
        world.insert(InputHandler::new());
        Ok(())
    }

    fn on_update(&mut self, event_bus: &EventBus, world: &mut specs::World) -> Result<()> {
        let mut input_handler = world.write_resource::<InputHandler>();
        input_handler.update(event_bus)
    }
}
