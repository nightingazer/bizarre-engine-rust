use anyhow::Result;
use bizarre_events::observer::EventBus;

use crate::{app_builder::AppBuilder, schedule::ScheduleBuilder};

pub trait Layer {
    fn on_attach(&mut self, app_builder: &mut AppBuilder) -> Result<()> {
        let _ = app_builder;
        Ok(())
    }

    #[deprecated]
    fn on_update(&mut self, world: &mut specs::World) -> Result<()> {
        let _ = world;
        Ok(())
    }
}
