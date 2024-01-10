use std::mem::offset_of;

use ash::vk;
use nalgebra_glm::{Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
    pub uv: Vec2,
}

impl Vertex {
    pub fn attribute_description() -> [vk::VertexInputAttributeDescription; 4] {
        [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex, position) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex, normal) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex, color) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(3)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex, uv) as u32)
                .build(),
        ]
    }
}

#[derive(Clone, Debug)]
pub struct Vertex2D {
    pub position: Vec2,
    pub color: Vec3,
    pub uv: Vec2,
}

#[repr(C)]
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
