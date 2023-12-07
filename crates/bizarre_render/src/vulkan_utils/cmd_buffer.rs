use ash::vk;

pub unsafe fn record_submit_cmd_buffer<F>(
    device: &ash::Device,
    cmd_buffer: vk::CommandBuffer,
    reuse_fence: vk::Fence,
    queue: vk::Queue,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphores: &[vk::Semaphore],
    f: F,
) where
    F: FnOnce(&ash::Device, vk::CommandBuffer),
{
    device
        .wait_for_fences(&[reuse_fence], true, u64::MAX)
        .expect("Wait for fence failed.");
    device
        .reset_fences(&[reuse_fence])
        .expect("Reset fence failed.");

    device
        .reset_command_buffer(cmd_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES)
        .expect("Reset command buffer failed.");

    let cmd_buffer_begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device
        .begin_command_buffer(cmd_buffer, &cmd_buffer_begin_info)
        .unwrap();

    f(device, cmd_buffer);

    device.end_command_buffer(cmd_buffer).unwrap();

    let cmd_buffers = [cmd_buffer];

    let submit_info = vk::SubmitInfo::builder()
        .wait_semaphores(wait_semaphores)
        .wait_dst_stage_mask(wait_mask)
        .command_buffers(&cmd_buffers)
        .signal_semaphores(signal_semaphores)
        .build();

    device
        .queue_submit(queue, &[submit_info], reuse_fence)
        .unwrap();
}
