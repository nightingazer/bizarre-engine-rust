use nalgebra_glm::Mat4;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Ubo {
    pub view_projection: Mat4,
    pub model: [Mat4; 100],
}
