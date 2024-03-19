use std::{default, marker::PhantomData, sync::Once};

use anyhow::Result;
use bizarre_logger::{
    core_critical, core_debug, core_info, global_loggers::logging_thread_start, logger_impl::Logger,
};
use specs::{World, WorldExt};

use crate::{
    app_events::AppCloseRequestedEvent,
    layer::Layer,
    schedule::{ScheduleBuilder, ScheduleType},
    App,
};

#[derive(Default, Clone)]
pub struct Yes;
#[derive(Default, Clone)]
pub struct No;

pub trait BuilderValidator {}

impl BuilderValidator for Yes {}
impl BuilderValidator for No {}

#[derive(Default)]
pub struct AppBuilder {
    pub name: Option<Box<str>>,
    pub schedule_builder: ScheduleBuilder,
    pub world: specs::World,
    pub layers: Vec<(Box<dyn Layer>, &'static str)>,
}

#[cfg(debug_assertions)]
static APP_BUILDER_ONCE: Once = Once::new();

impl AppBuilder {
    pub fn new() -> AppBuilder {
        #[cfg(debug_assertions)]
        {
            if APP_BUILDER_ONCE.is_completed() {
                panic!("Cannot build application more than once!");
            }
            APP_BUILDER_ONCE.call_once(|| {});
        }

        logging_thread_start(None);

        AppBuilder {
            world: specs::World::new(),
            ..Default::default()
        }
    }

    pub fn name(mut self, name: &str) -> AppBuilder {
        self.name = Some(name.into());
        self
    }

    pub fn with_layer<L>(mut self, layer: L) -> Self
    where
        L: Layer + 'static,
    {
        self.layers
            .push((Box::new(layer), std::any::type_name::<L>()));
        self
    }

    pub fn add_system<S>(
        &mut self,
        schedule_type: ScheduleType,
        system: S,
        name: &str,
        dependencies: &[&str],
    ) where
        S: for<'a> specs::System<'a> + 'static + Send,
    {
        self.schedule_builder
            .with_system(schedule_type, system, name, dependencies);
    }

    pub fn with_system<S>(
        mut self,
        schedule_type: ScheduleType,
        system: S,
        name: &str,
        dependencies: &[&str],
    ) -> Self
    where
        S: for<'a> specs::System<'a> + 'static + Send,
    {
        self.add_system(schedule_type, system, name, dependencies);
        self
    }

    pub fn add_barrier(&mut self, schedule_type: ScheduleType) {
        let dispatcher = match schedule_type {
            ScheduleType::Frame => self.schedule_builder.frame_dispatcher.as_mut().unwrap(),
            ScheduleType::Setup => self.schedule_builder.setup_dispatcher.as_mut().unwrap(),
        };
        dispatcher.add_barrier();
    }

    pub fn with_barrier(mut self, schedule_type: ScheduleType) -> Self {
        self.add_barrier(schedule_type);
        self
    }

    pub fn build(mut self) -> Result<App> {
        let name = self
            .name
            .take()
            .expect("Cannot create an app without a name");

        core_info!("Started the logger thread!");

        let layers = std::mem::take(&mut self.layers);

        for (mut layer, type_name) in layers {
            core_debug!("AppBuilder: attaching layer {}", type_name);
            if let Err(err) = layer.on_attach(&mut self) {
                let msg = format!("Failed to attach the layer: {err:?}");
                core_critical!(msg);
                panic!("{msg}");
            }
        }

        self.world.maintain();

        Ok(App {
            world: self.world,
            name,
            schedule: self.schedule_builder.build(),
            running: false,
            app_close_reader: None,
        })
    }
}
