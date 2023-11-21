use std::sync::Arc;

use anyhow::Result;
use vulkano::{device::Device, format::Format, render_pass::RenderPass, swapchain::Swapchain};

pub fn create_render_pass(
    swapchain: Arc<Swapchain>,
    device: Arc<Device>,
) -> Result<Arc<RenderPass>> {
    let render_pass = vulkano::ordered_passes_renderpass!(
        device.clone(),
        attachments: {
            final_color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            color: {
                format: Format::A2B10G10R10_UNORM_PACK32,
                samples: 1,
                load_op: Clear,
                store_op: DontCare,
            },
            normals: {
                format: Format::R16G16B16A16_SFLOAT,
                samples: 1,
                load_op: Clear,
                store_op: DontCare,
            },
            depth: {
                format: Format::D16_UNORM,
                samples: 1,
                load_op: Clear,
                store_op: DontCare,
            }
        },
        passes: [
            {
                color: [color, normals],
                depth_stencil: {depth},
                input: [],
            },
            {
                color: [final_color],
                depth_stencil: {},
                input: [color, normals],
            },
            {
                color: [final_color],
                depth_stencil: {depth},
                input: []
            },
            {
                color: [final_color],
                depth_stencil: {},
                input: []
            }
        ]
    )?;

    Ok(render_pass)
}
