use std::ops::Range;

use nalgebra_glm::Mat4;

use crate::{
    render_math::{AmbientLight, DirectionalLight},
    vertex::ColorNormalVertex,
};

#[derive(Clone, Debug)]
pub struct MeshSubmission {
    pub index_range: Range<u32>,
    pub model_matrix_offset: u32,
}

#[derive(Clone, Debug)]
pub struct RenderPackage {
    pub meshes: Vec<MeshSubmission>,
    pub model_matrices: [Mat4; 100],
    pub vertices: Vec<ColorNormalVertex>,
    pub indices: Vec<u32>,
    pub ambient_light: Option<AmbientLight>,
    pub directional_lights: Vec<DirectionalLight>,
    pub clear_color: [f32; 4],
    pub view: Mat4,
    pub projection: Mat4,
    pub view_projection_was_updated: bool,
}
