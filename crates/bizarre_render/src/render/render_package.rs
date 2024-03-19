use nalgebra_glm::{Mat4, Vec3};

use crate::{
    material::MaterialInstance, material_loader::MaterialInstanceHandle, mesh_loader::MeshHandle,
};

#[derive(Clone, Debug)]
pub struct MeshDelete {
    pub handle: MeshHandle,
}

#[derive(Clone, Debug)]
pub struct MeshUpload {
    pub mesh: MeshHandle,
}

#[derive(Clone)]
pub struct DrawSubmission {
    pub handle: MeshHandle,
    pub model_matrix: Mat4,
    pub material_instance: MaterialInstanceHandle,
}

#[derive(Clone)]
pub struct RenderPackage {
    pub mesh_uploads: Vec<MeshUpload>,
    pub mesh_deletes: Vec<MeshDelete>,
    pub draw_submissions: Vec<DrawSubmission>,

    pub view_projection: Mat4,
    pub view: Mat4,
    pub projection: Mat4,
    pub ambient_color: Vec3,

    pub avg_frame_time_ms: f64,
    pub last_frame_time_ms: f64,
}
