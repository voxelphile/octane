use crate::prelude::*;

use std::rc::Rc;

use raw_window_handle::HasRawWindowHandle;

pub struct SurfaceInfo<'a> {
    pub context: &'a Context,
    pub window: &'a dyn HasRawWindowHandle,
}

#[non_exhaustive]
pub enum Surface {
    Vulkan { surface: Rc<vk::Surface> },
}

impl Surface {
    pub fn new(info: SurfaceInfo) -> Self {
        match info.context {
            Context::Vulkan { instance, .. } => {
                let surface = vk::Surface::new(instance.clone(), &info.window);

                Self::Vulkan { surface }
            }
        }
    }

    pub(crate) fn get_vk_surface_format(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> vk::SurfaceFormat {
        vk::SurfaceFormat {
            format: vk::Format::Bgra8Srgb,
            color_space: vk::ColorSpace::SrgbNonlinear,
        }
    }
}
