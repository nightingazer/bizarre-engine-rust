use std::{
    ptr::{null, null_mut},
    sync::mpsc::{channel, Receiver},
    time::Duration,
};

use bizarre_events::observer::{EventBus, Observer};
use bizarre_logger::{core_debug, core_info, info};
use bizarre_render::renderer::{create_renderer, Renderer, RendererBackend};
use specs::WorldExt;

use crate::{app_events::AppCloseRequestedEvent, input::key_codes::KeyboardKey, layer::Layer};

pub struct App {
    name: Box<str>,
    event_bus: EventBus,
    world: specs::World,
    layers: Vec<Box<dyn Layer>>,
    observer: AppObserver,
    running: bool,
}

struct AppObserver {
    pub running: bool,
}

impl AppObserver {
    fn handle_close_request(&mut self, _: &AppCloseRequestedEvent) {
        core_info!("AppObserver: Got AppCloseRequestedEvent!");
        self.running = false;
    }
}

impl Observer for AppObserver {
    fn initialize(event_bus: &EventBus, system: bizarre_events::observer::SyncObserver<Self>) {
        event_bus.subscribe(system, Self::handle_close_request);
    }
}

impl App {
    pub fn new(name: &str) -> Self {
        let mut app = Self {
            name: name.into(),
            event_bus: EventBus::new(),
            world: specs::World::new(),
            layers: Vec::new(),
            observer: AppObserver { running: true },
            running: true,
        };

        app
    }

    pub fn run<'a>(&mut self) {
        core_info!("Running the \"{}\" application", self.name);

        self.event_bus.add_system(&mut self.observer);

        let (tx, rx) = channel();

        ctrlc::set_handler(move || {
            tx.send(AppCloseRequestedEvent {});
        });

        while self.observer.running {
            for layer in self.layers.iter_mut() {
                layer.on_update(&self.event_bus, &mut self.world);
            }

            if let Ok(event) = rx.try_recv() {
                self.event_bus.push_event(event);
            }
        }
    }

    fn destroy(&mut self) -> anyhow::Result<()> {
        core_info!("Destroying \"{}\" application", self.name);

        Ok(())
    }

    pub fn add_layer<L: Layer + 'static>(&mut self, mut layer: L) {
        layer.on_attach(&self.event_bus, &mut self.world);
        self.layers.push(Box::new(layer));
    }
}
