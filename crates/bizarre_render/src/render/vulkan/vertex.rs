use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

use crate::vertex::VertexData;

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
