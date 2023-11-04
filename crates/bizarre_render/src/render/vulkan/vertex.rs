use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[repr(C)]
#[derive(BufferContents, Vertex)]
pub struct VertexData {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
}

pub const CUBE_VERTICES: [VertexData; 36] = [
    // front face
    VertexData {
        position: [-1.000000, -1.000000, 1.000000],
        normal: [0.0000, 0.0000, 1.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, 1.000000, 1.000000],
        normal: [0.0000, 0.0000, 1.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, 1.000000, 1.000000],
        normal: [0.0000, 0.0000, 1.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, -1.000000, 1.000000],
        normal: [0.0000, 0.0000, 1.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, 1.000000, 1.000000],
        normal: [0.0000, 0.0000, 1.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, -1.000000, 1.000000],
        normal: [0.0000, 0.0000, 1.0000],
        color: [1.0, 0.35, 0.137],
    },
    // back face
    VertexData {
        position: [1.000000, -1.000000, -1.000000],
        normal: [0.0000, 0.0000, -1.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, 1.000000, -1.000000],
        normal: [0.0000, 0.0000, -1.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, 1.000000, -1.000000],
        normal: [0.0000, 0.0000, -1.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, -1.000000, -1.000000],
        normal: [0.0000, 0.0000, -1.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, 1.000000, -1.000000],
        normal: [0.0000, 0.0000, -1.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, -1.000000, -1.000000],
        normal: [0.0000, 0.0000, -1.0000],
        color: [1.0, 0.35, 0.137],
    },
    // top face
    VertexData {
        position: [-1.000000, -1.000000, 1.000000],
        normal: [0.0000, -1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, -1.000000, 1.000000],
        normal: [0.0000, -1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, -1.000000, -1.000000],
        normal: [0.0000, -1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, -1.000000, 1.000000],
        normal: [0.0000, -1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, -1.000000, -1.000000],
        normal: [0.0000, -1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, -1.000000, -1.000000],
        normal: [0.0000, -1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    // bottom face
    VertexData {
        position: [1.000000, 1.000000, 1.000000],
        normal: [0.0000, 1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, 1.000000, 1.000000],
        normal: [0.0000, 1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, 1.000000, -1.000000],
        normal: [0.0000, 1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, 1.000000, 1.000000],
        normal: [0.0000, 1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, 1.000000, -1.000000],
        normal: [0.0000, 1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, 1.000000, -1.000000],
        normal: [0.0000, 1.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    // left face
    VertexData {
        position: [-1.000000, -1.000000, -1.000000],
        normal: [-1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, 1.000000, -1.000000],
        normal: [-1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, 1.000000, 1.000000],
        normal: [-1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, -1.000000, -1.000000],
        normal: [-1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, 1.000000, 1.000000],
        normal: [-1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [-1.000000, -1.000000, 1.000000],
        normal: [-1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    // right face
    VertexData {
        position: [1.000000, -1.000000, 1.000000],
        normal: [1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, 1.000000, 1.000000],
        normal: [1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, 1.000000, -1.000000],
        normal: [1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, -1.000000, 1.000000],
        normal: [1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, 1.000000, -1.000000],
        normal: [1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
    VertexData {
        position: [1.000000, -1.000000, -1.000000],
        normal: [1.0000, 0.0000, 0.0000],
        color: [1.0, 0.35, 0.137],
    },
];
