use crate::prelude::*;

use raw_window_handle::HasRawWindowHandle;

pub struct SurfaceInfo<'a> {
    pub context: &'a Context,
    pub window: &'a dyn HasRawWindowHandle,
}

pub enum Surface {
    Vulkan {
        surface: vk::Surface,
        format: vk::SurfaceFormat,
    },
}

impl Surface {
    pub fn new(info: SurfaceInfo) -> Self {
        match info.context {
            Context::Vulkan { instance, .. } => {
                let surface = vk::Surface::new(instance.clone(), &info.window);

                let format = vk::SurfaceFormat {
                    format: vk::Format::Bgra8Srgb,
                    color_space: vk::ColorSpace::SrgbNonlinear,
                };

                Self::Vulkan { surface, format }
            }
        }
    }
}
