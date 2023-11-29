use std::{borrow::Cow, ffi::CStr, os::raw::c_char, sync::Arc};

use anyhow::{anyhow, Result};
use ash::{
    extensions::ext::DebugUtils,
    vk::{self, DebugUtilsMessengerCreateInfoEXT},
};
use bizarre_logger::{core_debug, core_error, core_info, core_warn};
use raw_window_handle::HasRawDisplayHandle;

use super::debug_utils::debug_messenger_create_info;

pub unsafe fn create_instance(
    window: Arc<winit::window::Window>,
    entry: &ash::Entry,
) -> Result<ash::Instance> {
    let app_name = CStr::from_bytes_with_nul_unchecked(b"Bizarre Engine App\0");
    let layer_names = [CStr::from_bytes_with_nul_unchecked(
        b"VK_LAYER_KHRONOS_validation\0",
    )];

    let layer_names_raw: Vec<*const c_char> = layer_names.iter().map(|l| l.as_ptr()).collect();
    let mut extensions_names =
        ash_window::enumerate_required_extensions(window.raw_display_handle())
            .unwrap()
            .to_vec();

    extensions_names.push(DebugUtils::name().as_ptr());

    let app_info = vk::ApplicationInfo::builder()
        .application_name(app_name)
        .application_version(0)
        .api_version(vk::make_api_version(0, 1, 0, 0))
        .engine_name(CStr::from_bytes_with_nul_unchecked(b"Bizarre Engine\0"))
        .engine_version(0);

    let create_flags = vk::InstanceCreateFlags::default();

    let mut debug_create_info = debug_messenger_create_info();

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layer_names_raw)
        .enabled_extension_names(&extensions_names)
        .flags(create_flags)
        .push_next(&mut debug_create_info);

    let instance = entry
        .create_instance(&create_info, None)
        .map_err(|e| anyhow!("Failed to create Vulkan instance: {}", e))?;

    Ok(instance)
}
