use std::ops::Range;

use nalgebra_glm::Mat4;

use crate::{
    mesh::Mesh,
    mesh_loader::MeshHandle,
    render_math::{AmbientLight, DirectionalLight},
    vertex::ColorNormalVertex,
};

#[derive(Clone, Debug)]
pub struct MeshDelete {
    pub handle: MeshHandle,
}

#[derive(Clone, Debug)]
pub struct MeshUpload {
    pub mesh: MeshHandle,
}

#[derive(Clone, Debug)]
pub struct DrawSubmission {
    pub handle: MeshHandle,
    pub model_matrix: Mat4,
}

#[derive(Clone, Debug)]
pub struct RenderPackage {
    pub mesh_uploads: Vec<MeshUpload>,
    pub mesh_deletes: Vec<MeshDelete>,
    pub draw_submissions: Vec<DrawSubmission>,

    pub avg_frame_time_ms: f64,
    pub last_frame_time_ms: f64,
}
