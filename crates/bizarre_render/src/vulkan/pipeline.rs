use ash::vk;

use crate::global_context::VULKAN_GLOBAL_CONTEXT;

pub struct VulkanPipeline {
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub set_layout: vk::DescriptorSetLayout,
}

impl VulkanPipeline {
    pub fn destroy(&mut self) {
        unsafe {
            let device = VULKAN_GLOBAL_CONTEXT.device();
            device.destroy_pipeline_layout(self.layout, None);
            self.layout = vk::PipelineLayout::null();
            device.destroy_descriptor_set_layout(self.set_layout, None);
            self.set_layout = vk::DescriptorSetLayout::null();
            device.destroy_pipeline(self.handle, None);
            self.handle = vk::Pipeline::null();
        }
    }
}
