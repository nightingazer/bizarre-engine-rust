use ash::vk;

pub struct VulkanPipeline {
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

impl VulkanPipeline {
    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_pipeline_layout(self.layout, None);
            self.layout = vk::PipelineLayout::null();
            device.destroy_pipeline(self.handle, None);
            self.handle = vk::Pipeline::null();
        }
    }
}
