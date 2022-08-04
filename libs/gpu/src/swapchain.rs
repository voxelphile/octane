use crate::prelude::*;

use std::rc::Rc;

pub struct SwapchainInfo<'a> {
    pub device: &'a Device,
    pub surface: &'a Surface,
    pub old: Option<Swapchain>,
}

pub struct SwapchainImageFetch<'a> {
    pub device: &'a Device,
    pub surface: &'a Surface,
}

#[non_exhaustive]
pub enum Swapchain {
    Vulkan {
        physical_device: Rc<vk::PhysicalDevice>,
        device: Rc<vk::Device>,
        surface: Rc<vk::Surface>,
        swapchain: vk::Swapchain,
    },
}

impl Swapchain {
    pub fn new(info: SwapchainInfo<'_>) -> Self {
        match info.device {
            Device::Vulkan {
                physical_device,
                device,
                ..
            } => {
                let surface = if let Surface::Vulkan { surface } = info.surface {
                    surface
                } else {
                    panic!("not a vulkan surface");
                };

                let vk::SurfaceCapabilities {
                    mut min_image_count,
                    current_transform: pre_transform,
                    current_extent: image_extent,
                    ..
                } = physical_device.surface_capabilities(&surface);

                min_image_count += 1;

                let vk::SurfaceFormat {
                    format: image_format,
                    color_space: image_color_space,
                } = physical_device.surface_format(&surface);

                let present_mode = vk::PresentMode::Immediate;

                let old_swapchain = info.old.map(|old| match old {
                    Self::Vulkan { swapchain, .. } => swapchain,
                    _ => panic!("not a vulkan swapchain"),
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

                Self::Vulkan {
                    physical_device: physical_device.clone(),
                    device: device.clone(),
                    surface: surface.clone(),
                    swapchain,
                }
            }
        }
    }

    pub fn images(&self) -> Vec<Image> {
        match self {
            Self::Vulkan {
                physical_device,
                device,
                surface,
                swapchain,
            } => {
                let vk::SurfaceFormat { format, .. } = physical_device.surface_format(&surface);

                swapchain
                    .images()
                    .into_iter()
                    .map(|image| {
                        let (view, sampler) = Image::new_vk_image_view(
                            device.clone(),
                            &image,
                            format,
                            vk::ImageViewType::TwoDim,
                        );

                        Image::Vulkan {
                            image,
                            format,
                            memory: None,
                            view,
                            sampler,
                        }
                    })
                    .collect::<Vec<_>>()
            }
        }
    }
}
