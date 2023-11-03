use bizarre_engine::{
    core::{input::key_codes::KeyboardKey, App},
    events::{
        event::Event,
        system::{EventBus, SyncSystem, System},
    },
    log::{app_logger_init, core_logger_init},
};

#[derive(Clone, Debug)]
struct SomeEvent {
    pub data: String,
}

impl Event for SomeEvent {}

struct SomeSystem {
    pub count: u32,
}

impl SomeSystem {
    fn handle_event(&mut self, event: &SomeEvent) {
        self.count += 1;
        println!("SomeSystem: {} (x{})", event.data, self.count);
    }
}

impl System for SomeSystem {
    fn initialize(event_bus: &EventBus, system: SyncSystem<Self>) {
        event_bus.subscribe(system, Self::handle_event);
    }
}

fn main() {
    // core_logger_init(None).expect("Failed to init core logger");
    // app_logger_init(None).expect("Failed to init app logger");

    // let mut app = App::default();
    // app.run();

    let event_bus = EventBus::new();

    let event = SomeEvent {
        data: "Hello, world!".into(),
    };

    let mut sys = SomeSystem { count: 0 };

    event_bus.push_event::<SomeEvent>(event.clone());
    event_bus.add_system(&mut sys);
    event_bus.push_event::<SomeEvent>(event.clone());
    event_bus.push_event::<SomeEvent>(event.clone());
}
