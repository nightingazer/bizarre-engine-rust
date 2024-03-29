#![feature(lazy_cell, slice_pattern, variant_count)]

mod assets;
mod render;
mod vulkan;
mod vulkan_shaders;

pub mod render_components;
pub mod render_systems;
pub mod vulkan_utils;

pub use assets::*;
pub use render::renderer::Renderer;
pub use render::*;
