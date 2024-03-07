use std::{
    sync::mpsc::{channel, TryRecvError},
    time::{Duration, Instant},
};

use bizarre_events::{
    event::EventQueue,
    observer::{EventBus, Observer},
};
use bizarre_logger::{core_critical, core_info, global_loggers::logging_thread_join};
use specs::WorldExt;

use crate::{
    app_builder::{AppBuilder, No},
    app_events::AppCloseRequestedEvent,
    debug_stats::DebugStats,
    layer::Layer,
    schedule::Schedule,
};

use bizarre_common::resources::{DeltaTime, RunningTime};

pub struct App {
    pub(crate) name: Box<str>,
    pub(crate) world: specs::World,
    pub(crate) schedule: Schedule,
    pub(crate) running: bool,
}

impl App {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            world: specs::World::new(),
            schedule: Schedule::default(),
            running: false,
        }
    }

    pub fn run(&mut self) {
        core_info!("Running the \"{}\" application", self.name);

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
        self.schedule
            .event_queues_update_dispatcher
            .setup(&mut self.world);

        self.running = true;

        while self.running {
            let frame_start = Instant::now();

            self.schedule.frame_dispatcher.dispatch(&self.world);

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

            {
                let mut close_event_queue = self
                    .world
                    .write_resource::<EventQueue<AppCloseRequestedEvent>>();

                match rx.try_recv() {
                    Ok(_) => close_event_queue.push_event(AppCloseRequestedEvent),
                    Err(err) => match err {
                        TryRecvError::Disconnected => self.running = false,
                        TryRecvError::Empty => {}
                    },
                }

                if !close_event_queue.get_events().is_empty() {
                    self.running = false;
                }
            }

            self.schedule
                .event_queues_update_dispatcher
                .dispatch(&self.world);

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
        self.running = false;
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }
}
