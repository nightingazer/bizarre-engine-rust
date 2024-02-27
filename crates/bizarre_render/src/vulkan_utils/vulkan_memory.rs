use ash::vk;

use crate::global_context::VULKAN_GLOBAL_CONTEXT;

pub fn find_memory_type_index(
    memory_req: &vk::MemoryRequirements,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    let memory_props = VULKAN_GLOBAL_CONTEXT.memory_properties();

    memory_props.memory_types[..memory_props.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(i, memory_type)| {
            (1 << i) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(i, _)| i as u32)
}
