use nalgebra_glm::Vec3;
use specs::{Component, VecStorage};

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct AmbientLight {
    pub color: Vec3,
    pub intensity: f32,
}

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
}

impl Component for DirectionalLight {
    type Storage = VecStorage<Self>;
}
