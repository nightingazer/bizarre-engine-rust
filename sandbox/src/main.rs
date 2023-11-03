use bizarre_engine::{
    core::{input::key_codes::KeyboardKey, App},
    events::{
        event::Event,
        observer::{EventBus, Observer, SyncObserver},
    },
    log::{app_logger_init, core_logger_init},
};

#[derive(Clone, Debug)]
struct SomeEvent {
    pub data: String,
}

impl Event for SomeEvent {}

struct SomeObserver {
    pub count: u32,
}

impl SomeObserver {
    fn handle_event(&mut self, event: &SomeEvent) {
        self.count += 1;
        println!("SomeSystem: {} (x{})", event.data, self.count);
    }
}

impl Observer for SomeObserver {
    fn initialize(event_bus: &EventBus, system: SyncObserver<Self>) {
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

    let mut sys = SomeObserver { count: 0 };

    event_bus.push_event::<SomeEvent>(event.clone());
    event_bus.add_system(&mut sys);
    event_bus.push_event::<SomeEvent>(event.clone());
    event_bus.push_event::<SomeEvent>(event.clone());
}
