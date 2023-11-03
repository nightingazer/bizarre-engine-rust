use bizarre_events::observer::EventBus;

use crate::App;

pub trait Layer {
    fn on_attach(&mut self, event_bus: &EventBus, world: &specs::World) {}
    fn on_update(&mut self, event_bus: &EventBus, world: &specs::World) {}
    fn on_detach(&mut self, event_bus: &EventBus, world: &specs::World) {}
}
