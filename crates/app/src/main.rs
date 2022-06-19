mod window;

use crate::window::{Event as WindowEvent, Window};

use log::{error, info, trace, warn};

fn debug_utils_messenger_callback(data: &vk::DebugUtilsMessengerCallbackData) -> bool {
    match data.message_severity {
        vk::DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE => trace!("{}", data.message),
        vk::DEBUG_UTILS_MESSAGE_SEVERITY_INFO => info!("{}", data.message),
        vk::DEBUG_UTILS_MESSAGE_SEVERITY_WARNING => warn!("{}", data.message),
        vk::DEBUG_UTILS_MESSAGE_SEVERITY_ERROR => error!("{}", data.message),
        _ => panic!("unrecognized message severity"),
    }

    false
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

//TODO identify why release segfaults
fn main() {
    println!("Hello, world!");

    log::set_max_level(log::LevelFilter::Trace);
    log::set_logger(&LOGGER);

    let mut window = Window::new();

    window.rename("Octane");
    window.show();

    let application_info = vk::ApplicationInfo {
        application_name: "Octane",
        application_version: (0, 1, 0).into(),
        engine_name: "Octane",
        engine_version: (0, 1, 0).into(),
        api_version: (1, 0, 0).into(),
    };

    let mut extensions = vec![vk::KHR_SURFACE, vk::KHR_XLIB_SURFACE];
    let mut layers = vec![];

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
    let debug_utils_messenger =
        vk::DebugUtilsMessenger::new(instance.clone(), debug_utils_messenger_create_info.unwrap())
            .expect("failed to create debug utils messenger");

    let physical_device = {
        let mut candidates = vk::PhysicalDevice::enumerate(instance.clone())
            .into_iter()
            .map(|x| (0, x.properties(), x)) // suitability of 0, pd properties, pd
            .collect::<Vec<_>>();

        if candidates.len() == 0 {
            panic!("no suitable gpu");
        }

        for (suitability, properties, _) in &mut candidates {
            if properties.device_type == vk::PhysicalDeviceType::Discrete {
                *suitability += 420;
            }

            *suitability += properties.limits.max_image_dimension_2d;

            trace!(
                "Found GPU \"{}\" with suitability of {}",
                properties.device_name,
                suitability
            );
        }

        candidates.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));

        let (_, properties, physical_device) = candidates.remove(0);

        info!("Selected GPU \"{}\"", properties.device_name);

        physical_device
    };

    let queue_families = physical_device.queue_families();

    let mut graphics_queue_family_index = 0;

    for (i, queue_family) in queue_families.iter().enumerate() {
        if queue_family.queue_flags & vk::QUEUE_GRAPHICS != 0 {
            graphics_queue_family_index = i as u32;
            break;
        }
    }

    let queue_create_info = vk::DeviceQueueCreateInfo {
        queue_family_index: graphics_queue_family_index,
        queue_priorities: &[1.0],
    };

    let physical_device_features = vk::PhysicalDeviceFeatures {};

    let device_create_info = vk::DeviceCreateInfo {
        queues: &[queue_create_info],
        enabled_features: &physical_device_features,
        extensions: &[],
        layers: &layers[..],
    };

    let device = vk::Device::new(physical_device, device_create_info)
        .expect("failed to create logical device");

    let queue = device.queue(graphics_queue_family_index);

    let surface = vk::Surface::new(instance.clone(), &window);

    loop {
        let event = window.next_event();

        match event {
            Some(WindowEvent::KeyPress) | Some(WindowEvent::CloseRequested) => {
                break;
            }
            None => {}
        }
    }

    //vk shutdown happens during implicit Drop.
    //Rc ensures shutdown happens in right order.
}
