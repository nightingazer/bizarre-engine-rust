use vulkano::buffer::BufferContents;

use crate::vertex::{ColorNormalVertex, PositionVertex, Vertex, Vertex2D};

#[repr(C)]
#[derive(BufferContents, vulkano::pipeline::graphics::vertex_input::Vertex)]
pub struct VulkanVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}

impl From<Vertex> for VulkanVertex {
    fn from(value: Vertex) -> Self {
        Self {
            position: value.position.into(),
            normal: value.normal.into(),
            uv: value.uv.into(),
        }
    }
}

#[repr(C)]
#[derive(BufferContents, vulkano::pipeline::graphics::vertex_input::Vertex)]
pub struct VulkanVertex2D {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2],
    #[format(R32G32B32_SFLOAT)]
    color: [f32; 3],
}

impl From<&Vertex2D> for VulkanVertex2D {
    fn from(value: &Vertex2D) -> Self {
        Self {
            position: value.position.into(),
            uv: value.uv.into(),
            color: value.color.into(),
        }
    }
}

#[repr(C)]
#[derive(BufferContents, vulkano::pipeline::graphics::vertex_input::Vertex)]
pub struct VulkanColorNormalVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
}

impl From<ColorNormalVertex> for VulkanColorNormalVertex {
    fn from(value: ColorNormalVertex) -> Self {
        Self {
            position: value.position.into(),
            color: value.color.into(),
            normal: value.normal.into(),
        }
    }
}

impl From<&ColorNormalVertex> for VulkanColorNormalVertex {
    fn from(value: &ColorNormalVertex) -> Self {
        Self {
            position: value.position.into(),
            color: value.color.into(),
            normal: value.normal.into(),
        }
    }
}

#[repr(C)]
#[derive(BufferContents, vulkano::pipeline::graphics::vertex_input::Vertex, Clone)]
pub struct VulkanPosition2DVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
}

impl VulkanPosition2DVertex {
    pub fn list() -> [VulkanPosition2DVertex; 6] {
        [
            VulkanPosition2DVertex {
                position: [-1.0, -1.0],
            },
            VulkanPosition2DVertex {
                position: [1.0, -1.0],
            },
            VulkanPosition2DVertex {
                position: [-1.0, 1.0],
            },
            VulkanPosition2DVertex {
                position: [1.0, -1.0],
            },
            VulkanPosition2DVertex {
                position: [1.0, 1.0],
            },
            VulkanPosition2DVertex {
                position: [-1.0, 1.0],
            },
        ]
    }
}

#[repr(C)]
#[derive(BufferContents, vulkano::pipeline::graphics::vertex_input::Vertex, Clone)]
pub struct VulkanPositionVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
}

impl From<PositionVertex> for VulkanPositionVertex {
    fn from(value: PositionVertex) -> Self {
        Self {
            position: value.position.into(),
        }
    }
}
