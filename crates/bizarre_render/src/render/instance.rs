use std::collections::HashSet;
use std::ffi::CStr;

use anyhow::anyhow;
use bizarre_logger::{core_debug, core_error, core_info, core_warn};
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_2::*;
use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::window as vk_window;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    ty: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        core_error!("{:?} {}", ty, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        core_warn!("{:?} {}", ty, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        core_info!("{:?} {}", ty, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE {
        core_debug!("{:?} {}", ty, message);
    }

    vk::FALSE
}

pub unsafe fn create_instance(
    window: &winit::window::Window,
    entry: &vulkanalia::Entry,
) -> anyhow::Result<(Instance, vk::DebugUtilsMessengerEXT)> {
    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Bizarre Engine\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"Bizarre Engine\0")
        .engine_version(vk::make_version(0, 1, 0))
        .api_version(vk::make_version(1, 2, 0));

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    if VALIDATION_ENABLED {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    let available_layers = entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();

    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!(
            "Validation layer requested but not available on the system"
        ));
    }

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        vec![]
    };

    let flags = vk::InstanceCreateFlags::empty();

    let mut debug_info: vk::DebugUtilsMessengerCreateInfoEXTBuilder = Default::default();
    let mut debug_messenger: vk::DebugUtilsMessengerEXT = Default::default();

    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_extension_names(&extensions)
        .enabled_layer_names(&layers)
        .flags(flags);

    if VALIDATION_ENABLED {
        debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .user_callback(Some(debug_callback));

        info = info.push_next(&mut debug_info);
    }

    let instance = entry.create_instance(&info, None)?;

    if VALIDATION_ENABLED {
        debug_messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok((instance, debug_messenger))
}
