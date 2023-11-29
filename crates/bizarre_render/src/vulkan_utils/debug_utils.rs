use std::{borrow::Cow, ffi::CStr};

use anyhow::Result;
use ash::{
    extensions::ext::DebugUtils,
    vk::{self, DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT},
};
use bizarre_logger::{core_debug, core_error, core_info, core_warn};

pub fn debug_messenger_create_info() -> DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(debug_msg_callback))
        .build()
}

pub unsafe fn create_debug_messenger(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> Result<DebugUtilsMessengerEXT> {
    let loader = DebugUtils::new(entry, instance);
    let create_info = debug_messenger_create_info();
    let messenger = loader.create_debug_utils_messenger(&create_info, None)?;

    Ok(messenger)
}

unsafe extern "system" fn debug_msg_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let msg_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    match severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            core_error!("Vulkan ({msg_type:?}): {msg_id_name} ({message_id_number}): {message}");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            core_warn!("Vulkan ({msg_type:?}): {msg_id_name} ({message_id_number}): {message}");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            core_info!("Vulkan ({msg_type:?}): {msg_id_name} ({message_id_number}): {message}");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            core_debug!("Vulkan ({msg_type:?}): {msg_id_name} ({message_id_number}): {message}");
        }
        _ => {}
    }

    vk::FALSE
}
