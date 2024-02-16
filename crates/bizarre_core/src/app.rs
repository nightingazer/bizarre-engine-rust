use std::{
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use anyhow::Result;
use bizarre_events::observer::{EventBus, Observer};
use bizarre_logger::{
    core_critical, core_info,
    global_loggers::{logging_thread_join, logging_thread_start},
};
use specs::WorldExt;

use crate::{
    app_builder::{AppBuilder, No},
    app_events::AppCloseRequestedEvent,
    debug_stats::DebugStats,
    layer::Layer,
    schedule::{Schedule, ScheduleBuilder},
};

use bizarre_common::resources::{DeltaTime, RunningTime};

pub struct App {
    pub(crate) name: Box<str>,
    pub(crate) event_bus: EventBus,
    pub(crate) world: specs::World,
    pub(crate) layers: Vec<Box<dyn Layer>>,
    pub(crate) observer: AppObserver,
    pub(crate) schedule: Schedule,
}

pub struct AppObserver {
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
        Self {
            name: name.into(),
            event_bus: EventBus::new(),
            world: specs::World::new(),
            layers: Vec::new(),
            observer: AppObserver { running: true },
            schedule: Schedule::default(),
        }
    }

    pub fn run(&mut self) {
        core_info!("Running the \"{}\" application", self.name);

        self.event_bus.add_observer(&mut self.observer);

        let (tx, rx) = channel::<AppCloseRequestedEvent>();

        {
            let tx = tx.clone();
            let result = ctrlc::set_handler(move || {
                let _ = tx.send(AppCloseRequestedEvent {});
            });
            if let Err(e) = result {
                core_critical!("Failed to set a termination handler: {}", e);
                self.destroy();
                return;
            }
        }

        self.world.insert(DeltaTime(Duration::from_secs(0)));
        self.world.insert(RunningTime(Duration::from_secs(0)));
        self.world.insert(DebugStats::default());

        self.schedule.frame_dispatcher.setup(&mut self.world);

        while self.observer.running {
            let frame_start = Instant::now();

            self.schedule.frame_dispatcher.dispatch(&self.world);

            for layer in self.layers.iter_mut() {
                let r = layer.on_update(&self.event_bus, &mut self.world);
                if let Err(e) = r {
                    core_critical!("Layer update failed: {0:?}", e);
                    self.destroy();
                    return;
                }
            }

            if let Ok(event) = rx.try_recv() {
                core_info!("Got a termination signal");
                self.event_bus.push_event(event);
            }

            let frame_duration = Instant::now() - frame_start;
            let sleep_duration = Duration::from_millis(16).saturating_sub(frame_duration);
            let delta_time = DeltaTime(frame_duration + sleep_duration);
            {
                let mut running_time = self.world.write_resource::<RunningTime>();
                let mut delta_time_res = self.world.write_resource::<DeltaTime>();

                *running_time = RunningTime(running_time.0 + delta_time.0);
                *delta_time_res = delta_time.clone();
                let mut debug_stats = self.world.write_resource::<DebugStats>();

                debug_stats.last_frame_work_time_ms = frame_duration.as_secs_f64() * 1000.0;
                debug_stats.last_frame_idle_time_ms = sleep_duration.as_secs_f64() * 1000.0;
                debug_stats.last_frame_total_time_ms = delta_time.0.as_secs_f64() * 1000.0;
            }

            self.world.maintain();

            if sleep_duration > Duration::from_millis(0) {
                std::thread::sleep(sleep_duration);
            }
        }

        self.destroy();

        logging_thread_join();
    }

    fn destroy(&mut self) {
        core_info!("Destroying \"{}\" application", self.name);

        for layer in self.layers.iter_mut() {
            layer.on_detach(&self.event_bus, &mut self.world);
        }
    }

    pub fn builder() -> AppBuilder<No> {
        AppBuilder::<No>::new()
    }
}
