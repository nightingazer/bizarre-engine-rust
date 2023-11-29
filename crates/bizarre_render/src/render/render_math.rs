use specs::{Component, VecStorage};

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct AmbientLight {
    pub color: [f32; 3],
    pub intensity: f32,
}

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct DirectionalLight {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Component for DirectionalLight {
    type Storage = VecStorage<Self>;
}
