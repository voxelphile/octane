use crate::prelude::*;

use std::rc::Rc;

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

        if self.contains(Self::TRANSFER_SRC) {
            vk |= vk::IMAGE_USAGE_TRANSFER_SRC;
        }

        if self.contains(Self::TRANSFER_DST) {
            vk |= vk::IMAGE_USAGE_TRANSFER_DST;
        }

        if self.contains(Self::SAMPLED) {
            vk |= vk::IMAGE_USAGE_SAMPLED;
        }

        if self.contains(Self::STORAGE) {
            vk |= vk::IMAGE_USAGE_STORAGE;
        }

        if self.contains(Self::COLOR) {
            vk |= vk::IMAGE_USAGE_COLOR_ATTACHMENT;
        }

        if self.contains(Self::DEPTH_STENCIL) {
            vk |= vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT;
        }

        if self.contains(Self::TRANSIENT) {
            vk |= vk::IMAGE_USAGE_TRANSIENT_ATTACHMENT;
        }

        if self.contains(Self::INPUT) {
            vk |= vk::IMAGE_USAGE_INPUT_ATTACHMENT;
        }

        vk
    }
}

#[derive(Clone, Copy)]
pub enum ImageLayout {
    Undefined,
    General,
    ColorAttachment,
    DepthStencilAttachment,
    DepthStencilReadOnly,
    ShaderReadOnly,
    TransferSrc,
    TransferDst,
    Preinitialized,
    PresentSrc,
}

impl From<ImageLayout> for vk::ImageLayout {
    fn from(layout: ImageLayout) -> Self {
        match layout {
            ImageLayout::Undefined => Self::Undefined,
            ImageLayout::General => Self::General,
            ImageLayout::ColorAttachment => Self::ColorAttachment,
            ImageLayout::DepthStencilAttachment => Self::DepthStencilAttachment,
            ImageLayout::DepthStencilReadOnly => Self::DepthStencilReadOnly,
            ImageLayout::ShaderReadOnly => Self::ShaderReadOnly,
            ImageLayout::TransferSrc => Self::TransferSrc,
            ImageLayout::TransferDst => Self::TransferDst,
            ImageLayout::Preinitialized => Self::Preinitialized,
            ImageLayout::PresentSrc => Self::PresentSrc,
        }
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

#[non_exhaustive]
pub enum Image {
    Vulkan {
        image: vk::Image,
        format: vk::Format,
        memory: Option<vk::Memory>,
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
                let (image, memory) = Self::new_managed_vk_image(
                    &physical_device,
                    device.clone(),
                    info.format.into(),
                    info.usage.to_vk(),
                    info.ty.to_vk_image(),
                    info.extent,
                );

                let (view, sampler) = Self::new_vk_image_view(
                    device.clone(),
                    &image,
                    info.format.into(),
                    info.ty.to_vk_image_view(),
                );

                Self::Vulkan {
                    image,
                    format: info.format.into(),
                    memory: Some(memory),
                    view,
                    sampler,
                }
            }
        }
    }

    pub(crate) fn new_managed_vk_image(
        physical_device: &vk::PhysicalDevice,
        device: Rc<vk::Device>,
        format: vk::Format,
        image_usage: u32,
        image_type: vk::ImageType,
        extent: (u32, u32, u32),
    ) -> (vk::Image, vk::Memory) {
        let image_create_info = vk::ImageCreateInfo {
            image_type,
            format,
            extent,
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SAMPLE_COUNT_1,
            tiling: vk::ImageTiling::Optimal,
            image_usage,
            initial_layout: vk::ImageLayout::Undefined,
        };

        let mut image =
            vk::Image::new(device.clone(), image_create_info).expect("failed to allocate image");

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

        (image, memory)
    }

    pub(crate) fn new_vk_image_view(
        device: Rc<vk::Device>,
        image: &vk::Image,
        format: vk::Format,
        view_type: vk::ImageViewType,
    ) -> (vk::ImageView, vk::Sampler) {
        let view_create_info = vk::ImageViewCreateInfo {
            image,
            view_type,
            format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::Identity,
                g: vk::ComponentSwizzle::Identity,
                b: vk::ComponentSwizzle::Identity,
                a: vk::ComponentSwizzle::Identity,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: format.aspect_mask(),
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

        (view, sampler)
    }
}
