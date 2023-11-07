use crate::{
    render_math::{AmbientLight, DirectionalLight},
    vertex::VertexData,
};

pub struct RenderPackage {
    pub vertices: Vec<VertexData>,
    pub ambient_light: AmbientLight,
    pub directional_lights: Vec<DirectionalLight>,
    pub clear_color: [f32; 4],
}
