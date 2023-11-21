use nalgebra_glm::{Vec2, Vec3};

#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
    pub uv: Vec2,
}

#[derive(Clone, Debug)]
pub struct Vertex2D {
    pub position: Vec2,
    pub color: Vec3,
    pub uv: Vec2,
}

#[derive(Clone, Debug)]
pub struct ColorNormalVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}

#[derive(Clone, Debug)]
pub struct PositionVertex {
    pub position: Vec3,
}
