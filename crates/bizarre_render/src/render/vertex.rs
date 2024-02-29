use std::mem::offset_of;

use ash::vk;
use nalgebra_glm::{Vec2, Vec3};

pub trait Vertex: Sized {
    fn attribute_description() -> Box<[vk::VertexInputAttributeDescription]>;
    fn binding_description() -> Box<[vk::VertexInputBindingDescription]> {
        Box::new([vk::VertexInputBindingDescription::builder()
            .binding(0)
            .input_rate(vk::VertexInputRate::VERTEX)
            .stride(std::mem::size_of::<Self>() as u32)
            .build()])
    }
}

impl Vertex for () {
    fn binding_description() -> Box<[vk::VertexInputBindingDescription]> {
        Box::new([])
    }

    fn attribute_description() -> Box<[vk::VertexInputAttributeDescription]> {
        Box::new([])
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct MeshVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec3,
    pub uv: Vec2,
}

impl Vertex for MeshVertex {
    fn attribute_description() -> Box<[vk::VertexInputAttributeDescription]> {
        Box::new([
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(MeshVertex, position) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(MeshVertex, normal) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(MeshVertex, color) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(3)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(MeshVertex, uv) as u32)
                .build(),
        ])
    }
}

#[repr(C)]
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

#[repr(C)]
#[derive(Clone, Debug)]
pub struct PositionVertex {
    pub position: Vec3,
}

impl Vertex for PositionVertex {
    fn attribute_description() -> Box<[vk::VertexInputAttributeDescription]> {
        Box::new([vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(PositionVertex, position) as u32)
            .build()])
    }
}
