use std::any::Any;

pub enum Event {
    /// Generic event used to relay arbitrary data between systems
    GenericEvent {
        data: Box<dyn Any>,
        name: String,
    },
    WindowEvent(),
    ApplicationEvent(),
    InputEvent(),
}

pub trait EventSubscriber {
    fn on_event(&mut self, event: &Event);
}
