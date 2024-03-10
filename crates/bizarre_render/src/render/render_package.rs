use nalgebra_glm::{Mat4, Vec3};

use crate::mesh_loader::MeshHandle;

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

    pub view_projection: Mat4,
    pub view: Mat4,
    pub projection: Mat4,

    pub avg_frame_time_ms: f64,
    pub last_frame_time_ms: f64,
}
