use bizarre_engine::{
    core::{
        app_events::AppCloseRequestedEvent, input::key_codes::KeyboardKey, layer::Layer, specs, App,
    },
    events::{
        event::Event,
        observer::{EventBus, Observer, SyncObserver},
    },
    layers::{input_layer::InputLayer, visual_layer::VisualLayer},
    log::{app_logger_init, core_logger_init, info},
};

struct KillerLayer {
    count: i32,
}

impl Layer for KillerLayer {
    fn on_attach(&mut self, event_bus: &EventBus, world: &mut specs::World) {
        info!("Attached KillerLayer!");
    }

    fn on_update(&mut self, event_bus: &EventBus, world: &mut specs::World) {
        self.count += 1;

        if self.count <= 10 {
            info!("KillerLayer: {}", self.count);
        }

        if self.count > 10 {
            event_bus.push_event(AppCloseRequestedEvent {});
        }
    }
}

fn main() {
    core_logger_init(None).expect("Failed to init core logger");
    app_logger_init(None).expect("Failed to init app logger");

    let mut app = App::new("Bizarre Engine");
    app.add_layer(InputLayer::new());
    app.add_layer(VisualLayer::new());
    app.run();
}
