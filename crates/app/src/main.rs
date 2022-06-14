mod window;

use crate::window::{Event as WindowEvent, Window};

fn main() {
    println!("Hello, world!");

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

    #[cfg(debug_assertions)]
    {
        extensions.push(vk::EXT_DEBUG_REPORT);
        layers.push(vk::LAYER_LUNARG_STANDARD_VALIDATION);
    }

    let create_info = vk::InstanceCreateInfo {
        application_info: &application_info,
        extensions: &extensions[..],
        layers: &layers[..],
    };

    let instance = vk::create_instance(create_info).expect("failed to create instance");

    loop {
        let event = window.next_event();

        match event {
            Some(WindowEvent::KeyPress) | Some(WindowEvent::CloseRequested) => {
                break;
            }
            None => {}
        }
    }
}
