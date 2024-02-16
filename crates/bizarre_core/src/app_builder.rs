use std::marker::PhantomData;

use anyhow::Result;
use bizarre_events::observer::EventBus;
use bizarre_logger::{core_info, global_loggers::logging_thread_start, logger_impl::Logger};
use specs::{World, WorldExt};

use crate::{
    layer::Layer,
    schedule::{ScheduleBuilder, ScheduleType},
    App, AppObserver,
};

#[derive(Default, Clone)]
pub struct Yes;
#[derive(Default, Clone)]
pub struct No;

pub trait BuilderValidator {}

impl BuilderValidator for Yes {}
impl BuilderValidator for No {}

#[derive(Default)]
pub struct AppBuilder<NameSet = No>
where
    NameSet: BuilderValidator,
{
    pub name: Option<Box<str>>,
    pub layers_to_add: Vec<Box<dyn Layer>>,
    pub schedule_builder: ScheduleBuilder,
    pub loggers_to_add: Vec<Logger>,
    _phantom_data: PhantomData<NameSet>,
}

impl<NameSet: BuilderValidator> AppBuilder<NameSet> {
    pub fn new() -> AppBuilder<No> {
        AppBuilder::<No>::default()
    }

    pub fn name(mut self, name: &str) -> AppBuilder<Yes> {
        self.name = Some(name.into());
        AppBuilder {
            layers_to_add: self.layers_to_add,
            loggers_to_add: self.loggers_to_add,
            schedule_builder: self.schedule_builder,
            name: self.name,
            ..Default::default()
        }
    }

    pub fn with_layer<L>(mut self, layer: L) -> Self
    where
        L: Layer + 'static,
    {
        self.layers_to_add.push(Box::new(layer));
        self
    }

    pub fn with_logger(mut self, logger: Logger) -> Self {
        self.loggers_to_add.push(logger);
        self
    }

    pub fn with_loggers<I>(mut self, loggers: I) -> Self
    where
        I: IntoIterator<Item = Logger>,
    {
        self.loggers_to_add.extend(loggers.into_iter());
        self
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
        match schedule_type {
            ScheduleType::Frame => {
                self.schedule_builder
                    .with_frame_system(system, name, dependencies)
            }
            _ => unimplemented!("It is possible to assign a system only to the frame schedule"),
        };

        self
    }
}

impl AppBuilder<Yes> {
    pub fn build(mut self) -> Result<App> {
        logging_thread_start(match self.loggers_to_add.len() {
            0 => None,
            _ => Some(self.loggers_to_add),
        });

        core_info!("Started the logger thread!");

        let event_bus = EventBus::new();
        let mut world = World::new();
        let layers = self
            .layers_to_add
            .into_iter()
            .map(|mut l| {
                l.on_attach(&event_bus, &mut world, &mut self.schedule_builder)?;
                Ok(l)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(App {
            event_bus,
            world,
            layers,
            name: self.name.unwrap(),
            observer: AppObserver { running: true },
            schedule: self.schedule_builder.build(),
        })
    }
}
