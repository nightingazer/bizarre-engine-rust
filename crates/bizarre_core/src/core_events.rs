use bizarre_events::event::Event;

#[derive(Debug, Clone)]
pub struct WindowResized {
    pub width: f32,
    pub height: f32,
}

impl Event for WindowResized {}
