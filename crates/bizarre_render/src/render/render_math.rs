use nalgebra_glm::{identity, TMat4};

pub struct ModelViewProjection {
    pub model: TMat4<f32>,
    pub view: TMat4<f32>,
    pub projection: TMat4<f32>,
}

impl Default for ModelViewProjection {
    fn default() -> Self {
        Self {
            model: identity(),
            view: identity(),
            projection: identity(),
        }
    }
}

impl ModelViewProjection {
    #[deprecated]
    pub fn new() -> Self {
        Self {
            model: identity(),
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
