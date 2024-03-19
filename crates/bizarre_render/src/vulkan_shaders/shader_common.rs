use nalgebra_glm::Mat4;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ViewProjection {
    pub view_projection: Mat4,
}
