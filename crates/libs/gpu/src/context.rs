use std::rc::Rc;

use log::{error, info, trace, warn};

#[non_exhaustive]
pub enum Context {
    Vulkan {
        instance: Rc<vk::Instance>,
        layers: Vec<&'static str>,
        extensions: Vec<&'static str>,
        #[cfg(debug_assertions)]
        debug: vk::DebugUtilsMessenger,
    },
}

fn debug_utils_messenger_callback(data: &vk::DebugUtilsMessengerCallbackData) -> bool {
    match data.message_severity {
        vk::DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE => trace!("{}\n", data.message),
        vk::DEBUG_UTILS_MESSAGE_SEVERITY_INFO => info!("{}\n", data.message),
        vk::DEBUG_UTILS_MESSAGE_SEVERITY_WARNING => warn!("{}\n", data.message),
        vk::DEBUG_UTILS_MESSAGE_SEVERITY_ERROR => error!("{}\n", data.message),
        _ => panic!("unrecognized message severity"),
    }

    false
}

impl Context {
    pub fn start() -> Self {
        Self::new_vulkan()
    }

    fn new_vulkan() -> Self {
        let application_info = vk::ApplicationInfo {
            application_name: "Octane",
            application_version: (0, 1, 0).into(),
            engine_name: "Octane",
            engine_version: (0, 1, 0).into(),
            api_version: (1, 0, 0).into(),
        };

        let mut extensions = vec![vk::KHR_SURFACE];
        let mut layers = vec![];

        #[cfg(target_os = "windows")]
        {
            extensions.push(vk::KHR_WIN32_SURFACE);
        }
        
        #[cfg(target_os = "linux")]
        {
            extensions.push(vk::KHR_XLIB_SURFACE);
        }

        let mut debug_utils_messenger_create_info = None;

        #[cfg(debug_assertions)]
        {
            extensions.push(vk::EXT_DEBUG_UTILS);
            layers.push(vk::LAYER_KHRONOS_VALIDATION);

            debug_utils_messenger_create_info = Some(vk::DebugUtilsMessengerCreateInfo {
                message_severity: vk::DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE
                    | vk::DEBUG_UTILS_MESSAGE_SEVERITY_INFO
                    | vk::DEBUG_UTILS_MESSAGE_SEVERITY_WARNING
                    | vk::DEBUG_UTILS_MESSAGE_SEVERITY_ERROR,
                message_type: vk::DEBUG_UTILS_MESSAGE_TYPE_GENERAL
                    | vk::DEBUG_UTILS_MESSAGE_TYPE_VALIDATION
                    | vk::DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE,
                user_callback: debug_utils_messenger_callback,
            });
        }

        let instance_create_info = vk::InstanceCreateInfo {
            application_info: &application_info,
            extensions: &extensions[..],
            layers: &layers[..],
            debug_utils: &debug_utils_messenger_create_info,
        };

        let instance = vk::Instance::new(instance_create_info).expect("failed to create instance");

        #[cfg(debug_assertions)]
        let debug = vk::DebugUtilsMessenger::new(
            instance.clone(),
            debug_utils_messenger_create_info.unwrap(),
        )
        .expect("failed to create debug utils messenger");

        Self::Vulkan {
            instance,
            layers,
            extensions,
            #[cfg(debug_assertions)]
            debug,
        }
    }
}
