use crate::prelude::*;

use std::collections::HashMap;
use std::iter;

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

bitflags! {
    #[repr(transparent)]
    pub struct CullMode: u32 {
        const FRONT = 0x00000001;
        const BACK = 0x00000002;
    }
}

impl CullMode {
    pub fn to_vk(self) -> u32 {
        self.bits()
    }
}

#[derive(Clone, Copy)]
pub enum CompareOp {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}

impl From<CompareOp> for vk::CompareOp {
    fn from(op: CompareOp) -> Self {
        match op {
            CompareOp::Never => Self::Never,
            CompareOp::Less => Self::Less,
            CompareOp::Equal => Self::Equal,
            CompareOp::LessOrEqual => Self::LessOrEqual,
            CompareOp::Greater => Self::Greater,
            CompareOp::NotEqual => Self::NotEqual,
            CompareOp::GreaterOrEqual => Self::GreaterOrEqual,
            CompareOp::Always => Self::Always,
        }
    }
}

#[derive(Clone, Copy)]
pub struct DepthStencil {
    pub test: bool,
    pub write: bool,
    pub compare_op: CompareOp,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InputRate {
    Vertex,
    Instance,
}

impl From<InputRate> for vk::VertexInputRate {
    fn from(input_rate: InputRate) -> Self {
        match input_rate {
            InputRate::Vertex => Self::Vertex,
            InputRate::Instance => Self::Instance,
        }
    }
}

#[derive(Clone, Copy)]
pub struct VertexInput {
    pub binding: u32,
    pub location: u32,
    pub format: Format,
    pub rate: InputRate,
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

#[derive(Clone, Copy)]
pub enum Binding<'a> {
    Buffer {
        binding: u32,
        ty: DescriptorType,
        offset: usize,
        range: usize,
        buffer: &'a Buffer,
    },
    Image {
        binding: u32,
        ty: DescriptorType,
        layout: ImageLayout,
        image: &'a Image,
    },
}

pub struct GraphicsPipelineInfo<'a> {
    pub device: &'a Device,
    pub render_pass: &'a RenderPass,
    pub descriptor_set_count: u32,
    pub color_count: u32,
    pub subpass: u32,
    pub extent: (u32, u32),
    pub cull_mode: CullMode,
    pub vertex_shader: &'a Shader,
    pub fragment_shader: Option<&'a Shader>,
    pub depth_stencil: DepthStencil,
    pub vertex_input: &'a [VertexInput],
    pub layout: &'a [Descriptor],
}

pub enum Pipeline {
    Vulkan {
        descriptor_sets: Vec<vk::DescriptorSet>,
        descriptor_set_layout: vk::DescriptorSetLayout,
        pipeline: vk::Pipeline,
        pipeline_layout: vk::PipelineLayout,
        bind_point: vk::PipelineBindPoint,
    },
}

impl Pipeline {
    pub fn new_graphics_pipeline(info: GraphicsPipelineInfo<'_>) -> Self {
        match info.device {
            Device::Vulkan {
                device,
                descriptor_pool,
                ..
            } => {
                let bindings = info
                    .layout
                    .iter()
                    .map(|descriptor| vk::DescriptorSetLayoutBinding {
                        binding: descriptor.binding,
                        descriptor_type: descriptor.ty.into(),
                        descriptor_count: descriptor.count,
                        stage: descriptor.stage.to_vk(),
                    })
                    .collect::<Vec<_>>();

                let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
                    bindings: &bindings,
                };

                let descriptor_set_layout =
                    vk::DescriptorSetLayout::new(device.clone(), descriptor_set_layout_create_info)
                        .expect("failed to create descriptor set layout");

                let set_layouts = iter::repeat(&descriptor_set_layout)
                    .take(info.descriptor_set_count as _)
                    .collect::<Vec<_>>();

                let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
                    descriptor_pool: &descriptor_pool,
                    set_layouts: &set_layouts,
                };

                let descriptor_sets =
                    vk::DescriptorSet::allocate(device.clone(), descriptor_set_allocate_info)
                        .expect("failed to allocate descriptor sets");

                let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
                    set_layouts: &[&descriptor_set_layout],
                };

                let pipeline_layout =
                    vk::PipelineLayout::new(device.clone(), pipeline_layout_create_info)
                        .expect("failed to create pipeline layout");

                let render_pass = if let RenderPass::Vulkan { render_pass } = info.render_pass {
                    render_pass
                } else {
                    panic!("not a vulkan surface");
                };

                let mut stages = vec![];

                let (vertex_module, vertex_entry) = if let Shader::Vulkan {
                    shader_module,
                    entry,
                    ..
                } = info.vertex_shader
                {
                    (shader_module, entry)
                } else {
                    panic!("not a vulkan shader");
                };

                let vertex_stage = vk::PipelineShaderStageCreateInfo {
                    stage: vk::SHADER_STAGE_VERTEX,
                    module: &vertex_module,
                    entry_point: &vertex_entry,
                };

                stages.push(vertex_stage);

                if let Some(shader) = info.fragment_shader {
                    if let Shader::Vulkan {
                        shader_module: fragment_module,
                        entry: fragment_entry,
                        ..
                    } = shader
                    {
                        let fragment_stage = vk::PipelineShaderStageCreateInfo {
                            stage: vk::SHADER_STAGE_FRAGMENT,
                            module: &fragment_module,
                            entry_point: &fragment_entry,
                        };

                        stages.push(fragment_stage);
                    } else {
                        panic!("not a vulkan shader");
                    }
                }

                let mut split = HashMap::<u32, Vec<VertexInput>>::new();

                info.vertex_input.iter().cloned().for_each(|input| {
                    split.entry(input.binding).or_default().push(input);
                });

                let mut bindings = vec![];
                let mut attributes = vec![];

                for (&binding, input) in &mut split {
                    input.sort_by(|a, b| a.location.cmp(&b.location));

                    if !input.iter().all(|i| i.rate == input[0].rate) {
                        panic!("all locations for a binding must have the same input rate")
                    }

                    let stride = input
                        .iter()
                        .map(|input| input.format.to_bytes())
                        .sum::<usize>();

                    let input_rate = input[0].rate.into();

                    let vertex_binding = vk::VertexInputBindingDescription {
                        binding,
                        stride,
                        input_rate,
                    };

                    bindings.push(vertex_binding);

                    for (x, i) in input.iter().enumerate() {
                        let location = i.location;

                        let format = i.format.into();

                        let offset = input
                            .iter()
                            .take(x)
                            .map(|i| i.format.to_bytes() as u32)
                            .sum::<u32>();

                        let vertex_attribute = vk::VertexInputAttributeDescription {
                            binding,
                            location,
                            format,
                            offset,
                        };

                        attributes.push(vertex_attribute);
                    }
                }

                let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
                    bindings: &bindings,
                    attributes: &attributes,
                };

                let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
                    topology: vk::PrimitiveTopology::TriangleList,
                    primitive_restart_enable: false,
                };

                let tessellation_state = vk::PipelineTessellationStateCreateInfo {};

                let viewport = vk::Viewport {
                    x: 0.0,
                    y: 0.0,
                    width: info.extent.0 as f32,
                    height: info.extent.1 as f32,
                    min_depth: 0.0,
                    max_depth: 1.0,
                };

                let scissor = vk::Rect2d {
                    offset: (0, 0),
                    extent: info.extent,
                };

                let viewport_state = vk::PipelineViewportStateCreateInfo {
                    viewports: &[viewport],
                    scissors: &[scissor],
                };

                let rasterizer = vk::PipelineRasterizationStateCreateInfo {
                    depth_clamp_enable: false,
                    rasterizer_discard_enable: false,
                    polygon_mode: vk::PolygonMode::Fill,
                    cull_mode: info.cull_mode.to_vk(),
                    front_face: vk::FrontFace::CounterClockwise,
                    depth_bias_enable: false,
                    depth_bias_constant_factor: 0.0,
                    depth_bias_clamp: 0.0,
                    depth_bias_slope_factor: 0.0,
                    line_width: 1.0,
                };

                let multisampling = vk::PipelineMultisampleStateCreateInfo {};

                let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
                    depth_test_enable: info.depth_stencil.test,
                    depth_write_enable: info.depth_stencil.write,
                    depth_compare_op: info.depth_stencil.compare_op.into(),
                    depth_bounds_test_enable: false,
                    min_depth_bounds: 0.0,
                    max_depth_bounds: 1.0,
                };

                let color_blend_attachments = (0..info.color_count)
                    .map(|_| vk::PipelineColorBlendAttachmentState {
                        color_write_mask: vk::COLOR_COMPONENT_R
                            | vk::COLOR_COMPONENT_G
                            | vk::COLOR_COMPONENT_B
                            | vk::COLOR_COMPONENT_A,
                        blend_enable: true,
                        src_color_blend_factor: vk::BlendFactor::SrcAlpha,
                        dst_color_blend_factor: vk::BlendFactor::OneMinusSrcAlpha,
                        color_blend_op: vk::BlendOp::Add,
                        src_alpha_blend_factor: vk::BlendFactor::SrcAlpha,
                        dst_alpha_blend_factor: vk::BlendFactor::OneMinusSrcAlpha,
                        alpha_blend_op: vk::BlendOp::Add,
                    })
                    .collect::<Vec<_>>();

                let color_blending = vk::PipelineColorBlendStateCreateInfo {
                    logic_op_enable: false,
                    logic_op: vk::LogicOp::Copy,
                    attachments: &color_blend_attachments[..],
                    blend_constants: &[0.0, 0.0, 0.0, 0.0],
                };

                let dynamic_state = vk::PipelineDynamicStateCreateInfo {
                    dynamic_states: &[],
                };

                let present_pipeline_create_info = vk::GraphicsPipelineCreateInfo {
                    stages: &stages,
                    vertex_input_state: &vertex_input_info,
                    input_assembly_state: &input_assembly,
                    tessellation_state: &tessellation_state,
                    viewport_state: &viewport_state,
                    rasterization_state: &rasterizer,
                    multisample_state: &multisampling,
                    depth_stencil_state: &depth_stencil,
                    color_blend_state: &color_blending,
                    dynamic_state: &dynamic_state,
                    layout: &pipeline_layout,
                    render_pass: &render_pass,
                    subpass: info.subpass,
                    base_pipeline: None,
                    base_pipeline_index: -1,
                };

                let pipeline = vk::Pipeline::new_graphics_pipelines(
                    device.clone(),
                    None,
                    &[present_pipeline_create_info],
                )
                .expect("failed to create graphics pipeline")
                .remove(0);

                Self::Vulkan {
                    descriptor_sets,
                    descriptor_set_layout,
                    pipeline,
                    pipeline_layout,
                    bind_point: vk::PipelineBindPoint::Graphics,
                }
            }
        }
    }

    pub fn bind(&mut self, image_index: u32, bindings: &'_ [Binding]) {
        match self {
            Pipeline::Vulkan {
                descriptor_sets, ..
            } => {
                let mut buffer_infos = vec![];
                let mut image_infos = vec![];

                let mut buffer_bindings = vec![];
                let mut image_bindings = vec![];

                let mut write_descriptors = vec![];

                for binding in bindings.clone() {
                    match binding {
                        Binding::Buffer {
                            offset,
                            range,
                            buffer,
                            ..
                        } => {
                            let buffer = if let Buffer::Vulkan { buffer, .. } = buffer {
                                buffer
                            } else {
                                panic!("not a vulkan buffer");
                            };

                            let buffer_info = vk::DescriptorBufferInfo {
                                buffer: &buffer,
                                offset: *offset as _,
                                range: *range as _,
                            };

                            let index = buffer_infos.len();

                            buffer_infos.push(buffer_info);

                            buffer_bindings.push(binding);
                        }
                        Binding::Image { layout, image, .. } => {
                            let (view, sampler) = if let Image::Vulkan { view, sampler, .. } = image
                            {
                                (view, sampler)
                            } else {
                                panic!("not a vulkan image");
                            };

                            let image_info = vk::DescriptorImageInfo {
                                sampler: &sampler,
                                image_view: &view,
                                image_layout: layout.clone().into(),
                            };

                            let index = image_infos.len();

                            image_infos.push(image_info);

                            image_bindings.push(binding);
                        }
                    }
                }

                for (index, binding) in buffer_bindings.iter().enumerate() {
                    let Binding::Buffer { binding, ty, .. } = binding else { panic!("not a vulkan buffer") };

                    let write_descriptor = vk::WriteDescriptorSet {
                        dst_set: &descriptor_sets[image_index as usize],
                        dst_binding: *binding,
                        dst_array_element: 0,
                        descriptor_count: 1,
                        descriptor_type: ty.clone().into(),
                        buffer_infos: &buffer_infos[index..=index],
                        image_infos: &[],
                    };

                    write_descriptors.push(write_descriptor);
                }

                for (index, binding) in image_bindings.iter().enumerate() {
                    let Binding::Image { binding, ty, .. } = binding else { panic!("not a vulkan image") };

                    let write_descriptor = vk::WriteDescriptorSet {
                        dst_set: &descriptor_sets[image_index as usize],
                        dst_binding: *binding,
                        dst_array_element: 0,
                        descriptor_count: 1,
                        descriptor_type: ty.clone().into(),
                        buffer_infos: &[],
                        image_infos: &image_infos[index..=index],
                    };

                    write_descriptors.push(write_descriptor);
                }

                vk::DescriptorSet::update(&write_descriptors, &[]);
            }
        }
    }
}
