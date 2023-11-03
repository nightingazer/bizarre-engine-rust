use bizarre_core::{input::input::InputHandler, layer::Layer, specs::WorldExt};
use bizarre_events::observer::EventBus;

pub struct InputLayer {}

impl InputLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Layer for InputLayer {
    fn on_attach(&mut self, event_bus: &EventBus, world: &mut bizarre_core::specs::World) {
        world.insert(InputHandler::new());
    }

    fn on_update(&mut self, event_bus: &EventBus, world: &mut bizarre_core::specs::World) {
        let mut input_handler = world.write_resource::<InputHandler>();
        input_handler.update();
    }
}
