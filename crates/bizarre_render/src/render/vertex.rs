use nalgebra_glm::Vec3;

#[derive(Clone)]
pub struct ColorNormalVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
}

#[derive(Clone)]
pub struct PositionVertex {
    pub position: [f32; 3],
}

pub const CUBE_MAP_VERTICES: [PositionVertex; 36] = [
    // front face
    PositionVertex {
        position: [-1.000000, -1.000000, 1.000000],
    },
    PositionVertex {
        position: [-1.000000, 1.000000, 1.000000],
    },
    PositionVertex {
        position: [1.000000, 1.000000, 1.000000],
    },
    PositionVertex {
        position: [-1.000000, -1.000000, 1.000000],
    },
    PositionVertex {
        position: [1.000000, 1.000000, 1.000000],
    },
    PositionVertex {
        position: [1.000000, -1.000000, 1.000000],
    },
    // back face
    PositionVertex {
        position: [1.000000, -1.000000, -1.000000],
    },
    PositionVertex {
        position: [1.000000, 1.000000, -1.000000],
    },
    PositionVertex {
        position: [-1.000000, 1.000000, -1.000000],
    },
    PositionVertex {
        position: [1.000000, -1.000000, -1.000000],
    },
    PositionVertex {
        position: [-1.000000, 1.000000, -1.000000],
    },
    PositionVertex {
        position: [-1.000000, -1.000000, -1.000000],
    },
    // top face
    PositionVertex {
        position: [-1.000000, -1.000000, 1.000000],
    },
    PositionVertex {
        position: [1.000000, -1.000000, 1.000000],
    },
    PositionVertex {
        position: [1.000000, -1.000000, -1.000000],
    },
    PositionVertex {
        position: [-1.000000, -1.000000, 1.000000],
    },
    PositionVertex {
        position: [1.000000, -1.000000, -1.000000],
    },
    PositionVertex {
        position: [-1.000000, -1.000000, -1.000000],
    },
    // bottom face
    PositionVertex {
        position: [1.000000, 1.000000, 1.000000],
    },
    PositionVertex {
        position: [-1.000000, 1.000000, 1.000000],
    },
    PositionVertex {
        position: [-1.000000, 1.000000, -1.000000],
    },
    PositionVertex {
        position: [1.000000, 1.000000, 1.000000],
    },
    PositionVertex {
        position: [-1.000000, 1.000000, -1.000000],
    },
    PositionVertex {
        position: [1.000000, 1.000000, -1.000000],
    },
    // left face
    PositionVertex {
        position: [-1.000000, -1.000000, -1.000000],
    },
    PositionVertex {
        position: [-1.000000, 1.000000, -1.000000],
    },
    PositionVertex {
        position: [-1.000000, 1.000000, 1.000000],
    },
    PositionVertex {
        position: [-1.000000, -1.000000, -1.000000],
    },
    PositionVertex {
        position: [-1.000000, 1.000000, 1.000000],
    },
    PositionVertex {
        position: [-1.000000, -1.000000, 1.000000],
    },
    // right face
    PositionVertex {
        position: [1.000000, -1.000000, 1.000000],
    },
    PositionVertex {
        position: [1.000000, 1.000000, 1.000000],
    },
    PositionVertex {
        position: [1.000000, 1.000000, -1.000000],
    },
    PositionVertex {
        position: [1.000000, -1.000000, 1.000000],
    },
    PositionVertex {
        position: [1.000000, 1.000000, -1.000000],
    },
    PositionVertex {
        position: [1.000000, -1.000000, -1.000000],
    },
];
