use specs::{prelude::*, Component};

#[derive(Debug, Component)]
pub struct WindowComponent {
    pub handle: winit::window::Window,
}
