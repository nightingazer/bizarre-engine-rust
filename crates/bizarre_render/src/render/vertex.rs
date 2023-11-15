use nalgebra_glm::Vec3;

#[derive(Clone)]
pub struct VertexData {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}

#[derive(Clone)]
pub struct PositionVertexData {
    pub position: [f32; 3],
}

pub const CUBE_MAP_VERTICES: [PositionVertexData; 36] = [
    // front face
    PositionVertexData {
        position: [-1.000000, -1.000000, 1.000000],
    },
    PositionVertexData {
        position: [-1.000000, 1.000000, 1.000000],
    },
    PositionVertexData {
        position: [1.000000, 1.000000, 1.000000],
    },
    PositionVertexData {
        position: [-1.000000, -1.000000, 1.000000],
    },
    PositionVertexData {
        position: [1.000000, 1.000000, 1.000000],
    },
    PositionVertexData {
        position: [1.000000, -1.000000, 1.000000],
    },
    // back face
    PositionVertexData {
        position: [1.000000, -1.000000, -1.000000],
    },
    PositionVertexData {
        position: [1.000000, 1.000000, -1.000000],
    },
    PositionVertexData {
        position: [-1.000000, 1.000000, -1.000000],
    },
    PositionVertexData {
        position: [1.000000, -1.000000, -1.000000],
    },
    PositionVertexData {
        position: [-1.000000, 1.000000, -1.000000],
    },
    PositionVertexData {
        position: [-1.000000, -1.000000, -1.000000],
    },
    // top face
    PositionVertexData {
        position: [-1.000000, -1.000000, 1.000000],
    },
    PositionVertexData {
        position: [1.000000, -1.000000, 1.000000],
    },
    PositionVertexData {
        position: [1.000000, -1.000000, -1.000000],
    },
    PositionVertexData {
        position: [-1.000000, -1.000000, 1.000000],
    },
    PositionVertexData {
        position: [1.000000, -1.000000, -1.000000],
    },
    PositionVertexData {
        position: [-1.000000, -1.000000, -1.000000],
    },
    // bottom face
    PositionVertexData {
        position: [1.000000, 1.000000, 1.000000],
    },
    PositionVertexData {
        position: [-1.000000, 1.000000, 1.000000],
    },
    PositionVertexData {
        position: [-1.000000, 1.000000, -1.000000],
    },
    PositionVertexData {
        position: [1.000000, 1.000000, 1.000000],
    },
    PositionVertexData {
        position: [-1.000000, 1.000000, -1.000000],
    },
    PositionVertexData {
        position: [1.000000, 1.000000, -1.000000],
    },
    // left face
    PositionVertexData {
        position: [-1.000000, -1.000000, -1.000000],
    },
    PositionVertexData {
        position: [-1.000000, 1.000000, -1.000000],
    },
    PositionVertexData {
        position: [-1.000000, 1.000000, 1.000000],
    },
    PositionVertexData {
        position: [-1.000000, -1.000000, -1.000000],
    },
    PositionVertexData {
        position: [-1.000000, 1.000000, 1.000000],
    },
    PositionVertexData {
        position: [-1.000000, -1.000000, 1.000000],
    },
    // right face
    PositionVertexData {
        position: [1.000000, -1.000000, 1.000000],
    },
    PositionVertexData {
        position: [1.000000, 1.000000, 1.000000],
    },
    PositionVertexData {
        position: [1.000000, 1.000000, -1.000000],
    },
    PositionVertexData {
        position: [1.000000, -1.000000, 1.000000],
    },
    PositionVertexData {
        position: [1.000000, 1.000000, -1.000000],
    },
    PositionVertexData {
        position: [1.000000, -1.000000, -1.000000],
    },
];
