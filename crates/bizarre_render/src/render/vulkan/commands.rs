use vulkanalia::prelude::v1_2::*;

use super::{
    devices::VulkanDevices, queue_families::QueueFamilyIndices,
    vulkan_renderer::VulkanRenderContext,
};

pub unsafe fn create_command_pool(
    instance: &Instance,
    devices: &VulkanDevices,
    surface: vk::SurfaceKHR,
) -> anyhow::Result<vk::CommandPool> {
    let indices = QueueFamilyIndices::get(instance, devices.physical, surface)?;
    let info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(indices.graphics)
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    let command_pool = devices.logical.create_command_pool(&info, None)?;

    Ok(command_pool)
}

pub unsafe fn create_command_buffers(
    buffer_count: u32,
    command_pool: vk::CommandPool,
    device: &Device,
) -> anyhow::Result<Vec<vk::CommandBuffer>> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(buffer_count);

    Ok(device.allocate_command_buffers(&allocate_info)?)
}

pub unsafe fn record_command_buffer(
    command_buffer: &vk::CommandBuffer,
    index: usize,
    ctx: &VulkanRenderContext,
) -> anyhow::Result<()> {
    let device = &ctx.devices.logical;

    let inheritance = vk::CommandBufferInheritanceInfo::builder();

    let info = vk::CommandBufferBeginInfo::builder()
        .inheritance_info(&inheritance)
        .flags(vk::CommandBufferUsageFlags::empty());

    device.begin_command_buffer(*command_buffer, &info)?;

    let render_area = vk::Rect2D::builder()
        .offset(vk::Offset2D::default())
        .extent(ctx.swapchain.extent);

    let color_clear_value = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        },
    };

    let clear_values = &[color_clear_value];

    let info = vk::RenderPassBeginInfo::builder()
        .render_pass(ctx.pipeline.render_pass)
        .framebuffer(ctx.framebuffers[index])
        .render_area(render_area)
        .clear_values(clear_values);

    device.cmd_begin_render_pass(*command_buffer, &info, vk::SubpassContents::INLINE);

    device.cmd_bind_pipeline(
        *command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        ctx.pipeline.pipeline,
    );

    device.cmd_draw(*command_buffer, 3, 1, 0, 0);

    device.cmd_end_render_pass(*command_buffer);
    Ok(device.end_command_buffer(*command_buffer)?)
}
