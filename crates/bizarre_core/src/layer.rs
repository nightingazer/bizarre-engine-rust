use anyhow::Result;
use bizarre_events::observer::EventBus;

use crate::schedule::ScheduleBuilder;

pub trait Layer {
    fn on_attach(
        &mut self,
        event_bus: &EventBus,
        world: &mut specs::World,
        schedule_builder: &mut ScheduleBuilder,
    ) -> Result<()> {
        let _ = world;
        let _ = event_bus;
        let _ = schedule_builder;
        Ok(())
    }

    fn on_update(&mut self, event_bus: &EventBus, world: &mut specs::World) -> Result<()> {
        let _ = event_bus;
        let _ = world;
        Ok(())
    }

    fn on_detach(&mut self, event_bus: &EventBus, world: &mut specs::World) {
        let _ = world;
        let _ = event_bus;
    }
}
