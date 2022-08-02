use crate::prelude::*;

pub struct SwapchainInfo<'a> {
    device: &'a Device,
    surface: &'a Surface,
    old: Option<Swapchain>,
}

pub enum Swapchain {
    Vulkan { swapchain: vk::Swapchain },
}

impl Swapchain {
    pub fn new(info: SwapchainInfo<'_>) -> Self {
        match info.device {
            Device::Vulkan {
                physical_device,
                device,
                ..
            } => {
                let (surface, surface_format) =
                    if let Surface::Vulkan { surface, format } = info.surface {
                        (surface, format)
                    } else {
                        panic!("surface must be vulkan if context is vulkan");
                    };

                let surface_capabilities = physical_device.surface_capabilities(&surface);

                let min_image_count = surface_capabilities.min_image_count + 1;

                let image_format = surface_format.format;

                let image_color_space = surface_format.color_space;

                let pre_transform = surface_capabilities.current_transform;

                let present_mode = vk::PresentMode::Immediate;

                let old_swapchain = info.old.map(|old| match old {
                    Self::Vulkan { swapchain } => swapchain,
                });

                let swapchain_create_info = vk::SwapchainCreateInfo {
                    surface,
                    min_image_count,
                    image_format,
                    image_color_space,
                    image_extent,
                    image_array_layers: 1,
                    image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT,
                    //TODO support concurrent image sharing mode
                    image_sharing_mode: vk::SharingMode::Exclusive,
                    queue_family_indices: &[],
                    pre_transform,
                    composite_alpha: vk::CompositeAlpha::Opaque,
                    present_mode,
                    clipped: true,
                    old_swapchain,
                };

                let mut swapchain = vk::Swapchain::new(device.clone(), swapchain_create_info)
                    .expect("failed to create swapchain");

                let swapchain_images = swapchain.images();

                Self::Vulkan { swapchain }
            }
        }
    }
}
