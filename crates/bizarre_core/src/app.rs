use std::{
    sync::mpsc::{channel, TryRecvError},
    time::{Duration, Instant},
};

use bizarre_logger::{core_critical, core_info, global_loggers::logging_thread_join};
use specs::{shrev::EventChannel, ReaderId, WorldExt};

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
    pub(crate) app_close_reader: Option<ReaderId<AppCloseRequestedEvent>>,
}

impl App {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            world: specs::World::new(),
            schedule: Schedule::default(),
            running: false,
            app_close_reader: None,
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

        {
            let mut event_channel = EventChannel::<AppCloseRequestedEvent>::new();
            self.app_close_reader = Some(event_channel.register_reader());

            self.world.insert(event_channel);
        }

        self.world.insert(DeltaTime(Duration::from_secs(0)));
        self.world.insert(RunningTime(Duration::from_secs(0)));
        self.world.insert(DebugStats::default());

        self.schedule.setup_dispatcher.setup(&mut self.world);
        self.schedule.setup_dispatcher.dispatch(&self.world);
        self.world.maintain();

        self.schedule.frame_dispatcher.setup(&mut self.world);

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
                match rx.try_recv() {
                    Ok(event) => {
                        let mut channel = self
                            .world
                            .write_resource::<EventChannel<AppCloseRequestedEvent>>();
                        channel.single_write(event)
                    }
                    Err(err) => match err {
                        TryRecvError::Disconnected => self.running = false,
                        TryRecvError::Empty => {}
                    },
                }
            }

            self.world.maintain();

            {
                let events = self
                    .world
                    .read_resource::<EventChannel<AppCloseRequestedEvent>>()
                    .read(self.app_close_reader.as_mut().unwrap())
                    .cloned()
                    .collect::<Vec<_>>();

                if !events.is_empty() {
                    self.running = false;
                    break;
                }
            }

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
