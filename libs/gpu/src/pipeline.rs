use crate::prelude::*;

use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    pub struct PipelineStage: u32 {
        const TOP_OF_PIPE = 0x00000001;
        const FRAGMENT_SHADER = 0x00000080;
        const EARLY_FRAGMENT_TESTS = 0x00000100;
        const LATE_FRAGMENT_TESTS = 0x00000200;
        const COLOR_ATTACHMENT_OUTPUT = 0x00000400;
        const COMPUTE_SHADER = 0x00000800;
        const TRANSFER = 0x00001000;
        const BOTTOM_OF_PIPE = 0x00002000;
    }
}

impl PipelineStage {
    pub fn to_vk(self) -> u32 {
        self.bits()
    }
}

#[derive(Clone, Copy)]
pub enum DescriptorType {
    CombinedImageSampler,
    StorageImage,
    UniformBuffer,
    StorageBuffer,
}

impl From<DescriptorType> for vk::DescriptorType {
    fn from(ty: DescriptorType) -> Self {
        match ty {
            DescriptorType::CombinedImageSampler => Self::CombinedImageSampler,
            DescriptorType::StorageImage => Self::StorageImage,
            DescriptorType::UniformBuffer => Self::UniformBuffer,
            DescriptorType::StorageBuffer => Self::StorageBuffer,
        }
    }
}

pub struct Descriptor {
    pub binding: u32,
    pub ty: DescriptorType,
    pub count: u32,
    pub stage: ShaderStage,
}

pub struct GraphicsPipelineInfo<'a> {
    pub device: &'a Device,
    pub vertex_shader: &'a Shader,
    pub fragment_shader: Option<&'a Shader>,
    pub layout: &'a [Descriptor],
}

pub enum Pipeline {
    Vulkan {
        descriptor_set: vk::DescriptorSet,
        descriptor_set_layout: vk::DescriptorSetLayout,
        pipeline: vk::Pipeline,
    },
}

impl Pipeline {
    pub fn new_graphics_pipeline(info: GraphicsPipelineInfo<'_>) -> Self {
        match info.device {
            Device::Vulkan { device, descriptor_pool, ..) => {
                
            }
        }
    }
}
