use bizarre_events::event::Event;

#[derive(Debug, Clone)]
pub struct AppCloseRequestedEvent;

impl Event for AppCloseRequestedEvent {}
