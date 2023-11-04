use std::{
    boxed,
    ptr::{null, null_mut},
    sync::mpsc::{channel, Receiver},
    time::{Duration, Instant},
};

use bizarre_events::observer::{EventBus, Observer};
use bizarre_logger::{core_debug, core_info, info};
use bizarre_render::renderer::{create_renderer, Renderer, RendererBackend};
use specs::WorldExt;

use crate::{
    app_events::AppCloseRequestedEvent,
    input::key_codes::KeyboardKey,
    layer::Layer,
    timing::{DeltaTime, RunningTime},
};

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

        let mut frame_start = Instant::now();

        self.world.insert(DeltaTime(0.0));
        self.world.insert(RunningTime(0.0));

        while self.observer.running {
            frame_start = Instant::now();

            for layer in self.layers.iter_mut() {
                layer.on_update(&self.event_bus, &mut self.world);
            }

            if let Ok(event) = rx.try_recv() {
                self.event_bus.push_event(event);
            }

            let mut running_time = self.world.write_resource::<RunningTime>();
            let mut delta_time_res = self.world.write_resource::<DeltaTime>();

            let frame_duration = Instant::now() - frame_start;
            let sleep_duration = Duration::from_millis(16).saturating_sub(frame_duration);
            let delta_time = DeltaTime(frame_duration.as_secs_f32() + sleep_duration.as_secs_f32());
            *running_time = RunningTime(running_time.0 + delta_time.0);
            *delta_time_res = delta_time;

            if sleep_duration > Duration::from_millis(0) {
                std::thread::sleep(sleep_duration);
            }
        }
    }

    fn destroy(&mut self) -> anyhow::Result<()> {
        core_info!("Destroying \"{}\" application", self.name);

        Ok(())
    }

    pub fn add_layer<L: Layer + 'static>(&mut self, mut layer: L) {
        let mut boxed = Box::new(layer);
        boxed.on_attach(&self.event_bus, &mut self.world);
        self.layers.push(boxed);
    }
}
