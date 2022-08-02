use crate::prelude::*;

use bitflags::bitflags;

bitflags! {
    pub struct ImageUsage: usize {
        const TRANSFER_SRC = 1 << 0;
        const TRANSFER_DST = 1 << 1;
        const SAMPLED = 1 << 2;
        const STORAGE = 1 << 3;
        const COLOR = 1 << 4;
        const DEPTH_STENCIL = 1 << 5;
        const TRANSIENT = 1 << 6;
        const INPUT = 1 << 7;
    }
}

impl ImageUsage {
    pub(crate) fn to_vk(self) -> u32 {
        let mut vk = 0;

        if self == Self::TRANSFER_SRC {
            vk |= vk::IMAGE_USAGE_TRANSFER_SRC;
        }

        if self == Self::TRANSFER_DST {
            vk |= vk::IMAGE_USAGE_TRANSFER_DST;
        }

        if self == Self::SAMPLED {
            vk |= vk::IMAGE_USAGE_SAMPLED;
        }

        if self == Self::STORAGE {
            vk |= vk::IMAGE_USAGE_STORAGE;
        }

        if self == Self::COLOR {
            vk |= vk::IMAGE_USAGE_COLOR_ATTACHMENT;
        }

        if self == Self::DEPTH_STENCIL {
            vk |= vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT;
        }

        if self == Self::TRANSIENT {
            vk |= vk::IMAGE_USAGE_TRANSIENT_ATTACHMENT;
        }

        if self == Self::INPUT {
            vk |= vk::IMAGE_USAGE_INPUT_ATTACHMENT;
        }

        vk
    }
}

#[derive(Clone, Copy)]
pub enum ImageType {
    OneDim,
    TwoDim,
    ThreeDim,
}

impl ImageType {
    pub(crate) fn to_vk_image(self) -> vk::ImageType {
        match self {
            Self::OneDim => vk::ImageType::OneDim,
            Self::TwoDim => vk::ImageType::TwoDim,
            Self::ThreeDim => vk::ImageType::ThreeDim,
        }
    }

    pub(crate) fn to_vk_image_view(self) -> vk::ImageViewType {
        match self {
            Self::OneDim => vk::ImageViewType::OneDim,
            Self::TwoDim => vk::ImageViewType::TwoDim,
            Self::ThreeDim => vk::ImageViewType::ThreeDim,
        }
    }
}

pub struct ImageInfo<'a> {
    pub device: &'a Device,
    pub format: Format,
    pub usage: ImageUsage,
    pub ty: ImageType,
    pub extent: (u32, u32, u32),
}

pub enum Image {
    Vulkan {
        image: vk::Image,
        memory: vk::Memory,
        view: vk::ImageView,
        sampler: vk::Sampler,
    },
}

impl Image {
    pub fn new(info: ImageInfo<'_>) -> Self {
        match info.device {
            Device::Vulkan {
                physical_device,
                device,
                ..
            } => {
                let image_create_info = vk::ImageCreateInfo {
                    image_type: info.ty.to_vk_image(),
                    format: info.format.to_vk(),
                    extent: info.extent,
                    mip_levels: 1,
                    array_layers: 1,
                    samples: vk::SAMPLE_COUNT_1,
                    tiling: vk::ImageTiling::Optimal,
                    image_usage: info.usage.to_vk(),
                    initial_layout: vk::ImageLayout::Undefined,
                };

                let mut image = vk::Image::new(device.clone(), image_create_info)
                    .expect("failed to allocate image");

                let memory_allocate_info = vk::MemoryAllocateInfo {
                    property_flags: vk::MEMORY_PROPERTY_DEVICE_LOCAL,
                };

                let memory = vk::Memory::allocate(
                    device.clone(),
                    memory_allocate_info,
                    image.memory_requirements(),
                    physical_device.memory_properties(),
                    false,
                )
                .expect("failed to allocate memory");

                image
                    .bind_memory(&memory)
                    .expect("failed to bind memory to image");

                let view_create_info = vk::ImageViewCreateInfo {
                    image: &image,
                    view_type: info.ty.to_vk_image_view(),
                    format: info.format.to_vk(),
                    components: vk::ComponentMapping {
                        r: vk::ComponentSwizzle::Identity,
                        g: vk::ComponentSwizzle::Identity,
                        b: vk::ComponentSwizzle::Identity,
                        a: vk::ComponentSwizzle::Identity,
                    },
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::IMAGE_ASPECT_COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };

                let view = vk::ImageView::new(device.clone(), view_create_info)
                    .expect("failed to create image view");

                let sampler_create_info = vk::SamplerCreateInfo {
                    mag_filter: vk::Filter::Nearest,
                    min_filter: vk::Filter::Nearest,
                    mipmap_mode: vk::SamplerMipmapMode::Nearest,
                    address_mode_u: vk::SamplerAddressMode::ClampToBorder,
                    address_mode_v: vk::SamplerAddressMode::ClampToBorder,
                    address_mode_w: vk::SamplerAddressMode::ClampToBorder,
                    mip_lod_bias: 0.0,
                    anisotropy_enable: false,
                    max_anisotropy: 0.0,
                    compare_enable: false,
                    compare_op: vk::CompareOp::Always,
                    min_lod: 0.0,
                    max_lod: 0.0,
                    border_color: vk::BorderColor::IntTransparentBlack,
                    unnormalized_coordinates: false,
                };

                let sampler = vk::Sampler::new(device.clone(), sampler_create_info)
                    .expect("failed to create sampler");

                Self::Vulkan {
                    image,
                    memory,
                    view,
                    sampler,
                }
            }
        }
    }
}
