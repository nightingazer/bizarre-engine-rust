use bizarre_events::event::Event;

pub struct WindowResized {
    pub width: f32,
    pub height: f32,
}

impl Event for WindowResized {}
