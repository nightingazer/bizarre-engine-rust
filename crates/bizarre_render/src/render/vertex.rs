use nalgebra_glm::{Vec2, Vec3};

#[derive(Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

#[derive(Clone)]
pub struct ColorNormalVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}

#[derive(Clone)]
pub struct PositionVertex {
    pub position: Vec3,
}
