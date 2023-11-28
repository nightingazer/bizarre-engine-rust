use std::sync::Arc;

use vulkano::{
    buffer::Subbuffer, descriptor_set::PersistentDescriptorSet, render_pass::Framebuffer,
};

use super::{shaders::deferred_vert, vulkan_text::VulkanScreenTextObject};

pub struct VulkanFrame {
    pub frame_index: u32,
    pub framebuffer: Arc<Framebuffer>,
    pub deferred_set: Arc<PersistentDescriptorSet>,
    pub deferred_ubo: Subbuffer<deferred_vert::UBO>,
    pub rt_text_obj: Option<VulkanScreenTextObject>,
    pub rt_text_set: Option<Arc<PersistentDescriptorSet>>,
}
