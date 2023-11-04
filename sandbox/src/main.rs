use bizarre_engine::{
    core::{
        app_events::AppCloseRequestedEvent,
        input::{input_event::InputEvent, key_codes::KeyboardKey},
        layer::Layer,
        specs, App,
    },
    events::{
        event::Event,
        observer::{EventBus, Observer, SyncObserver},
    },
    layers::{input_layer::InputLayer, visual_layer::VisualLayer},
    log::{app_logger_init, core_logger_init, info},
};

struct InputObserverLayer {}

impl InputObserverLayer {
    fn handle_input_event(&mut self, event: &InputEvent) {
        info!("InputObserverLayer: Got InputEvent: {:?}", event);
    }
}

impl Observer for InputObserverLayer {
    fn initialize(event_bus: &EventBus, system: SyncObserver<Self>) {
        event_bus.subscribe(system, Self::handle_input_event);
    }
}

impl Layer for InputObserverLayer {
    fn on_attach(&mut self, event_bus: &EventBus, _: &mut specs::World) {
        event_bus.add_system(self);
    }
}

fn main() {
    core_logger_init(None).expect("Failed to init core logger");
    app_logger_init(None).expect("Failed to init app logger");

    let mut app = App::new("Bizarre Engine");
    app.add_layer(InputLayer::new());
    app.add_layer(VisualLayer::new());
    app.add_layer(InputObserverLayer {});
    app.run();
}
