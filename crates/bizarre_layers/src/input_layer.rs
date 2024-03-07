use anyhow::Result;
use bizarre_core::{
    app_builder::AppBuilder,
    input::{InputHandler, KeyboardEvent, MouseEvent},
    layer::Layer,
    schedule::{ScheduleBuilder, ScheduleType},
};
use specs::{shrev::EventChannel, System, WorldExt, Write};

use crate::visual_layer::WinitEventSystem;

#[derive(Default)]
pub struct InputLayer;

pub struct InputHandlerUpdate;

impl InputHandlerUpdate {
    pub const DEFAULT_NAME: &'static str = "input_handler_update";
}

impl<'a> System<'a> for InputHandlerUpdate {
    type SystemData = (
        Write<'a, InputHandler>,
        Write<'a, EventChannel<MouseEvent>>,
        Write<'a, EventChannel<KeyboardEvent>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut handler, mut mouse, mut keyboard) = data;
        handler.update(&mut (&mut mouse, &mut keyboard));
    }
}

impl Layer for InputLayer {
    fn on_attach(&mut self, app_builder: &mut AppBuilder) -> Result<()> {
        app_builder.world.insert(InputHandler::default());

        app_builder.add_system(
            ScheduleType::Frame,
            InputHandlerUpdate,
            InputHandlerUpdate::DEFAULT_NAME,
            &[WinitEventSystem::DEFAULT_NAME],
        );

        Ok(())
    }
}
