use std::{default, marker::PhantomData, sync::Once};

use anyhow::Result;
use bizarre_events::{
    event::{Event, EventQueue, EventQueueUpdateSystem},
    observer::EventBus,
};
use bizarre_logger::{core_info, global_loggers::logging_thread_start, logger_impl::Logger};
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

    pub fn with_layer<L>(mut self, mut layer: L) -> Self
    where
        L: Layer + 'static,
    {
        layer.on_attach(&mut self);
        self
    }

    pub fn with_event<E>(mut self) -> Self
    where
        E: Event,
    {
        self.add_event::<E>();
        self
    }

    pub fn add_event<E: Event>(&mut self) {
        self.world.insert(EventQueue::<E>::default());
        self.schedule_builder
            .with_event_cleaner(EventQueueUpdateSystem::<E>::default());
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
        match schedule_type {
            ScheduleType::Frame => {
                self.schedule_builder
                    .with_frame_system(system, name, dependencies)
            }
            _ => unimplemented!("It is possible to assign a system only to the frame schedule"),
        };
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

    pub fn build(mut self) -> Result<App> {
        let name = self
            .name
            .take()
            .expect("Cannot create an app without a name");

        core_info!("Started the logger thread!");

        self.add_event::<AppCloseRequestedEvent>();

        Ok(App {
            world: self.world,
            name,
            schedule: self.schedule_builder.build(),
            running: false,
        })
    }
}
