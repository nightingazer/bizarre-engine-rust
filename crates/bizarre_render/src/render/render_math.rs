use nalgebra_glm::{identity, TMat4};
use vulkano::buffer::BufferContents;

#[repr(C)]
#[derive(BufferContents)]
pub struct ModelViewProjection {
    pub model: TMat4<f32>,
    pub view: TMat4<f32>,
    pub projection: TMat4<f32>,
}

impl ModelViewProjection {
    pub fn new() -> Self {
        Self {
            model: identity(),
            view: identity(),
            projection: identity(),
        }
    }
}
