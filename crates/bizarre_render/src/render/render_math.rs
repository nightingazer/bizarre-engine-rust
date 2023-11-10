use nalgebra_glm::{identity, TMat4};
use specs::{Component, VecStorage};

pub struct ViewProjection {
    pub view: TMat4<f32>,
    pub projection: TMat4<f32>,
}

impl Default for ViewProjection {
    fn default() -> Self {
        Self {
            view: identity(),
            projection: identity(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct AmbientLight {
    pub color: [f32; 3],
    pub intensity: f32,
}

#[derive(Default, Debug, Clone)]
pub struct DirectionalLight {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Component for DirectionalLight {
    type Storage = VecStorage<Self>;
}
