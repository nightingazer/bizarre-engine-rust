use nalgebra_glm::Mat4;

use crate::{
    render_math::{AmbientLight, DirectionalLight},
    vertex::VertexData,
};

pub struct RenderPackage {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub ambient_light: Option<AmbientLight>,
    pub directional_lights: Vec<DirectionalLight>,
    pub clear_color: [f32; 4],
    pub view_projection: Option<Mat4>,
}
