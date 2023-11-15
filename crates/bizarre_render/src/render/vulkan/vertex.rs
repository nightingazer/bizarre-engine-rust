use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

use crate::vertex::{PositionVertexData, VertexData};

#[repr(C)]
#[derive(BufferContents, Vertex)]
pub struct VulkanVertexData {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
}

impl From<VertexData> for VulkanVertexData {
    fn from(value: VertexData) -> Self {
        Self {
            position: value.position.into(),
            color: value.color.into(),
            normal: value.normal.into(),
        }
    }
}

#[repr(C)]
#[derive(BufferContents, Vertex, Clone)]
pub struct DummyVertexData {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
}

impl DummyVertexData {
    pub fn list() -> [DummyVertexData; 6] {
        [
            DummyVertexData {
                position: [-1.0, -1.0],
            },
            DummyVertexData {
                position: [1.0, -1.0],
            },
            DummyVertexData {
                position: [-1.0, 1.0],
            },
            DummyVertexData {
                position: [1.0, -1.0],
            },
            DummyVertexData {
                position: [1.0, 1.0],
            },
            DummyVertexData {
                position: [-1.0, 1.0],
            },
        ]
    }
}

#[repr(C)]
#[derive(BufferContents, Vertex, Clone)]
pub struct VulkanPositionVertexData {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
}

impl From<PositionVertexData> for VulkanPositionVertexData {
    fn from(value: PositionVertexData) -> Self {
        Self {
            position: value.position.into(),
        }
    }
}
