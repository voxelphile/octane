use crate::bucket::Bucket;
use crate::mesh::{Mesh, Vertex};
use crate::octree::{Node, Octree};

use math::prelude::{Matrix, Vector};

use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::iter;
use std::mem;
use std::rc::Rc;
use std::time;

use log::{error, info, trace, warn};
use raw_window_handle::HasRawWindowHandle;

pub const CHUNK_SIZE: usize = 8;
static mut JFAI_DONE: bool = true;
//temporary for here for now.
#[derive(Default, Clone, Copy)]
pub struct Camera {
    pub model: Matrix<f32, 4, 4>,
    pub view: Matrix<f32, 4, 4>,
    pub proj: Matrix<f32, 4, 4>,
    pub camera: Matrix<f32, 4, 4>,
}

#[derive(Default, Clone, Copy)]
pub struct RenderSettings {
    pub resolution: Vector<f32, 2>,
    pub render_distance: u32,
}

pub struct RendererInfo<'a> {
    pub window: &'a dyn HasRawWindowHandle,
    pub render_distance: u32,
    pub hq4x: String,
}

pub trait Renderer {
    fn draw_batch(&mut self, batch: Batch, entries: &'_ [Entry<'_>]);
    fn resize(&mut self, resolution: (u32, u32));
}

#[derive(Clone, Default)]
pub struct Batch {
    pub present_vertex_shader: String,
    pub present_fragment_shader: String,
    pub postfx_vertex_shader: String,
    pub postfx_fragment_shader: String,
    pub graphics_vertex_shader: String,
    pub graphics_fragment_shader: String,
    pub jfa_shader: String,
}

#[derive(Clone, Copy)]
pub struct Entry<'a> {
    pub mesh: &'a Mesh,
}

fn convert_bytes_to_spirv_data(bytes: Vec<u8>) -> Vec<u32> {
    let endian = mem::size_of::<u32>() / mem::size_of::<u8>();

    if bytes.len() % endian != 0 {
        panic!("cannot convert bytes to int; too few or too many")
    }

    let mut buffer = Vec::with_capacity(bytes.len() / endian);

    for slice in bytes.chunks(endian) {
        buffer.push(u32::from_le_bytes(slice.try_into().unwrap()));
    }

    buffer
}

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

fn create_compute_pipeline(
    device: Rc<vk::Device>,
    stage: vk::PipelineShaderStageCreateInfo<'_>,
    layout: &'_ vk::PipelineLayout,
) -> vk::Pipeline {
    let compute_pipeline_create_info = vk::ComputePipelineCreateInfo {
        stage,
        layout,
        base_pipeline: None,
        base_pipeline_index: -1,
    };

    vk::Pipeline::new_compute_pipelines(device, None, &[compute_pipeline_create_info])
        .expect("failed to create compute pipeline")
        .remove(0)
}

fn create_graphics_pipeline(
    device: Rc<vk::Device>,
    vertex_input_info: vk::PipelineVertexInputStateCreateInfo,
    stages: &'_ [vk::PipelineShaderStageCreateInfo<'_>],
    render_pass: &'_ vk::RenderPass,
    layout: &'_ vk::PipelineLayout,
    extent: (u32, u32),
    attachment_count: usize,
    cull_mode: u32,
) -> vk::Pipeline {
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TriangleList,
        primitive_restart_enable: false,
    };

    let tessellation_state = vk::PipelineTessellationStateCreateInfo {};

    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: extent.0 as f32,
        height: extent.1 as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    };

    let scissor = vk::Rect2d {
        offset: (0, 0),
        extent,
    };

    let viewport_state = vk::PipelineViewportStateCreateInfo {
        viewports: &[viewport],
        scissors: &[scissor],
    };

    let rasterizer = vk::PipelineRasterizationStateCreateInfo {
        depth_clamp_enable: false,
        rasterizer_discard_enable: false,
        polygon_mode: vk::PolygonMode::Fill,
        cull_mode,
        front_face: vk::FrontFace::CounterClockwise,
        depth_bias_enable: false,
        depth_bias_constant_factor: 0.0,
        depth_bias_clamp: 0.0,
        depth_bias_slope_factor: 0.0,
        line_width: 1.0,
    };

    let multisampling = vk::PipelineMultisampleStateCreateInfo {};

    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
        depth_test_enable: true,
        depth_write_enable: true,
        depth_compare_op: vk::CompareOp::Less,
        depth_bounds_test_enable: false,
        min_depth_bounds: 0.0,
        max_depth_bounds: 1.0,
    };

    let color_blend_attachments = (0..attachment_count)
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
        stages,
        vertex_input_state: &vertex_input_info,
        input_assembly_state: &input_assembly,
        tessellation_state: &tessellation_state,
        viewport_state: &viewport_state,
        rasterization_state: &rasterizer,
        multisample_state: &multisampling,
        depth_stencil_state: &depth_stencil,
        color_blend_state: &color_blending,
        dynamic_state: &dynamic_state,
        layout: &layout,
        render_pass: &render_pass,
        subpass: 0,
        base_pipeline: None,
        base_pipeline_index: -1,
    };

    vk::Pipeline::new_graphics_pipelines(device, None, &[present_pipeline_create_info])
        .expect("failed to create graphics pipeline")
        .remove(0)
}

pub struct Vulkan {
    pub camera: Bucket<Camera>,
    pub settings: Bucket<RenderSettings>,
    last_camera: Option<Camera>,
    octree: Octree,
    last_batch: Batch,
    look_up_table_sampler: vk::Sampler,
    look_up_table_view: vk::ImageView,
    look_up_table_memory: vk::Memory,
    look_up_table: vk::Image,
    /*cubelet_sdf_result_sampler: vk::Sampler,
    cubelet_sdf_result_view: vk::ImageView,
    cubelet_sdf_result_memory: vk::Memory,
    cubelet_sdf_result: vk::Image,*/
    octree_buffer_memory: vk::Memory,
    octree_buffer: vk::Buffer,
    instance_data: Vec<Vector<u32, 3>>,
    instance_buffer_memory: vk::Memory,
    instance_buffer: vk::Buffer,
    data_buffer_memory: vk::Memory,
    data_buffer: vk::Buffer,
    staging_buffer_memory: vk::Memory,
    staging_buffer: vk::Buffer,
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
    command_buffer: vk::CommandBuffer,
    command_pool: vk::CommandPool,
    render_info: VulkanRenderInfo,
    render_data: Option<VulkanRenderData>,
    compute_data: Option<VulkanComputeData>,
    queue: vk::Queue,
    device: Rc<vk::Device>,
    physical_device: vk::PhysicalDevice,
    shaders: HashMap<String, vk::ShaderModule>,
    shader_mod_time: HashMap<String, time::SystemTime>,
    surface: vk::Surface,
    #[cfg(debug_assertions)]
    debug_utils_messenger: vk::DebugUtilsMessenger,
    pub instance: Rc<vk::Instance>,
}

pub struct VulkanRenderInfo {
    image_count: u32,
    surface_format: vk::SurfaceFormat,
    surface_capabilities: vk::SurfaceCapabilities,
    present_mode: vk::PresentMode,
    extent: (u32, u32),
    scaling_factor: u32,
}

pub struct VulkanComputeData {
    jfa_pipeline: vk::Pipeline,
    jfa_pipeline_layout: vk::PipelineLayout,
    jfa_descriptor_sets: Vec<vk::DescriptorSet>,
    jfa_descriptor_pool: vk::DescriptorPool,
    jfa_descriptor_set_layout: vk::DescriptorSetLayout,
}

impl VulkanComputeData {
    pub fn init(device: Rc<vk::Device>, jfa_stage: vk::PipelineShaderStageCreateInfo<'_>) -> Self {
        let uniform_buffer_binding = vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_COMPUTE,
        };

        let octree_buffer_binding = vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::StorageBuffer,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_COMPUTE,
        };

        /*let cubelet_sdf_result_binding = vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_COMPUTE,
        };
        */

        let jfai_buffer_binding = vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::StorageBuffer,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_COMPUTE,
        };

        let jfa_descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            bindings: &[
                uniform_buffer_binding,
                octree_buffer_binding,
                //      cubelet_sdf_result_binding,
                jfai_buffer_binding,
            ],
        };

        let jfa_descriptor_set_layout =
            vk::DescriptorSetLayout::new(device.clone(), jfa_descriptor_set_layout_create_info)
                .expect("failed to create descriptor set layout");

        let uniform_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: 1,
        };

        let octree_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageBuffer,
            descriptor_count: 1,
        };
        /*
                let cubelet_sdf_result_pool_size = vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::StorageImage,
                    descriptor_count: 1,
                };
        */
        let jfai_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageBuffer,
            descriptor_count: 1,
        };

        let jfa_descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            max_sets: 1,
            pool_sizes: &[
                uniform_buffer_pool_size,
                octree_buffer_pool_size,
                //              cubelet_sdf_result_pool_size,
                jfai_buffer_pool_size,
            ],
        };

        let jfa_descriptor_pool =
            vk::DescriptorPool::new(device.clone(), jfa_descriptor_pool_create_info)
                .expect("failed to create descriptor pool");

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: &jfa_descriptor_pool,
            set_layouts: &[&jfa_descriptor_set_layout],
        };

        let jfa_descriptor_sets =
            vk::DescriptorSet::allocate(device.clone(), descriptor_set_allocate_info)
                .expect("failed to allocate descriptor sets");

        let jfa_pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            set_layouts: &[&jfa_descriptor_set_layout],
        };

        let jfa_pipeline_layout =
            vk::PipelineLayout::new(device.clone(), jfa_pipeline_layout_create_info)
                .expect("failed to create pipeline layout");

        let jfa_pipeline = create_compute_pipeline(device.clone(), jfa_stage, &jfa_pipeline_layout);

        Self {
            jfa_pipeline,
            jfa_pipeline_layout,
            jfa_descriptor_sets,
            jfa_descriptor_pool,
            jfa_descriptor_set_layout,
        }
    }
}

pub struct VulkanRenderData {
    graphics_color_samplers: Vec<vk::Sampler>,
    graphics_color_views: Vec<vk::ImageView>,
    graphics_color_memory: Vec<vk::Memory>,
    graphics_color: Vec<vk::Image>,
    graphics_occlusion_samplers: Vec<vk::Sampler>,
    graphics_occlusion_views: Vec<vk::ImageView>,
    graphics_occlusion_memory: Vec<vk::Memory>,
    graphics_occlusion: Vec<vk::Image>,
    graphics_framebuffers: Vec<vk::Framebuffer>,
    graphics_pipeline: vk::Pipeline,
    graphics_pipeline_layout: vk::PipelineLayout,
    graphics_descriptor_sets: Vec<vk::DescriptorSet>,
    graphics_descriptor_pool: vk::DescriptorPool,
    graphics_descriptor_set_layout: vk::DescriptorSetLayout,
    graphics_render_pass: vk::RenderPass,
    postfx_color_samplers: Vec<vk::Sampler>,
    postfx_color_views: Vec<vk::ImageView>,
    postfx_color_memory: Vec<vk::Memory>,
    postfx_color: Vec<vk::Image>,
    postfx_framebuffers: Vec<vk::Framebuffer>,
    postfx_pipeline: vk::Pipeline,
    postfx_pipeline_layout: vk::PipelineLayout,
    postfx_descriptor_sets: Vec<vk::DescriptorSet>,
    postfx_descriptor_pool: vk::DescriptorPool,
    postfx_descriptor_set_layout: vk::DescriptorSetLayout,
    postfx_render_pass: vk::RenderPass,
    present_framebuffers: Vec<vk::Framebuffer>,
    present_pipeline: vk::Pipeline,
    present_pipeline_layout: vk::PipelineLayout,
    present_descriptor_sets: Vec<vk::DescriptorSet>,
    present_descriptor_pool: vk::DescriptorPool,
    present_descriptor_set_layout: vk::DescriptorSetLayout,
    present_render_pass: vk::RenderPass,
    distance_samplers: Vec<vk::Sampler>,
    distance_views: Vec<vk::ImageView>,
    distance_memory: Vec<vk::Memory>,
    distance: Vec<vk::Image>,
    depth_sampler: vk::Sampler,
    depth_view: vk::ImageView,
    depth_memory: vk::Memory,
    depth: vk::Image,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain: vk::Swapchain,
}

impl VulkanRenderData {
    pub fn init(
        device: Rc<vk::Device>,
        physical_device: &vk::PhysicalDevice,
        surface: &vk::Surface,
        graphics_shader_stages: &'_ [vk::PipelineShaderStageCreateInfo<'_>],
        postfx_shader_stages: &'_ [vk::PipelineShaderStageCreateInfo<'_>],
        present_shader_stages: &'_ [vk::PipelineShaderStageCreateInfo<'_>],
        old_swapchain: Option<vk::Swapchain>,
        render_info: &VulkanRenderInfo,
    ) -> Self {
        //DEPTH
        let depth_create_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TwoDim,
            format: vk::Format::D32Sfloat,
            extent: (render_info.extent.0, render_info.extent.1, 1),
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SAMPLE_COUNT_1,
            tiling: vk::ImageTiling::Optimal,
            image_usage: vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT,
            initial_layout: vk::ImageLayout::Undefined,
        };

        let mut depth =
            vk::Image::new(device.clone(), depth_create_info).expect("failed to allocate image");

        let depth_memory_allocate_info = vk::MemoryAllocateInfo {
            property_flags: vk::MEMORY_PROPERTY_DEVICE_LOCAL,
        };

        let depth_memory = vk::Memory::allocate(
            device.clone(),
            depth_memory_allocate_info,
            depth.memory_requirements(),
            physical_device.memory_properties(),
            false,
        )
        .expect("failed to allocate memory");

        depth
            .bind_memory(&depth_memory)
            .expect("failed to bind image to memory");

        let depth_view_create_info = vk::ImageViewCreateInfo {
            image: &depth,
            view_type: vk::ImageViewType::TwoDim,
            format: vk::Format::D32Sfloat,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::Identity,
                g: vk::ComponentSwizzle::Identity,
                b: vk::ComponentSwizzle::Identity,
                a: vk::ComponentSwizzle::Identity,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::IMAGE_ASPECT_DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        };

        let depth_view = vk::ImageView::new(device.clone(), depth_view_create_info)
            .expect("failed to create image view");

        let depth_sampler = {
            let depth_sampler_create_info = vk::SamplerCreateInfo {
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

            vk::Sampler::new(device.clone(), depth_sampler_create_info)
                .expect("failed to create sampler")
        };

        //SWAPCHAIN
        let swapchain_create_info = vk::SwapchainCreateInfo {
            surface,
            min_image_count: render_info.image_count,
            image_format: render_info.surface_format.format,
            image_color_space: render_info.surface_format.color_space,
            image_extent: render_info.extent,
            image_array_layers: 1,
            image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT,
            //TODO support concurrent image sharing mode
            image_sharing_mode: vk::SharingMode::Exclusive,
            queue_family_indices: &[],
            pre_transform: render_info.surface_capabilities.current_transform,
            composite_alpha: vk::CompositeAlpha::Opaque,
            present_mode: render_info.present_mode,
            clipped: true,
            old_swapchain,
        };

        let mut swapchain = vk::Swapchain::new(device.clone(), swapchain_create_info)
            .expect("failed to create swapchain");

        let swapchain_images = swapchain.images();

        let swapchain_image_views = swapchain_images
            .iter()
            .map(|image| {
                let create_info = vk::ImageViewCreateInfo {
                    image,
                    view_type: vk::ImageViewType::TwoDim,
                    format: render_info.surface_format.format,
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

                vk::ImageView::new(device.clone(), create_info)
                    .expect("failed to create image view")
            })
            .collect::<Vec<_>>();

        //DISTANCE
        let mut distance = (0..swapchain_images.len())
            .map(|_| {
                let distance_create_info = vk::ImageCreateInfo {
                    image_type: vk::ImageType::TwoDim,
                    format: vk::Format::Rgba32Sfloat,
                    extent: (
                        render_info.extent.0 / render_info.scaling_factor,
                        render_info.extent.1 / render_info.scaling_factor,
                        1,
                    ),
                    mip_levels: 1,
                    array_layers: 1,
                    samples: vk::SAMPLE_COUNT_1,
                    tiling: vk::ImageTiling::Optimal,
                    image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT | vk::IMAGE_USAGE_STORAGE,
                    initial_layout: vk::ImageLayout::Undefined,
                };

                vk::Image::new(device.clone(), distance_create_info)
                    .expect("failed to allocate image")
            })
            .collect::<Vec<_>>();

        let distance_memory = distance
            .iter_mut()
            .map(|distance| {
                let distance_memory_allocate_info = vk::MemoryAllocateInfo {
                    property_flags: vk::MEMORY_PROPERTY_DEVICE_LOCAL,
                };

                let distance_memory = vk::Memory::allocate(
                    device.clone(),
                    distance_memory_allocate_info,
                    distance.memory_requirements(),
                    physical_device.memory_properties(),
                    false,
                )
                .expect("failed to allocate memory");

                distance
                    .bind_memory(&distance_memory)
                    .expect("failed to bind image to memory");

                distance_memory
            })
            .collect::<Vec<_>>();

        let distance_views = distance
            .iter()
            .map(|distance| {
                let distance_view_create_info = vk::ImageViewCreateInfo {
                    image: distance,
                    view_type: vk::ImageViewType::TwoDim,
                    format: vk::Format::Rgba32Sfloat,
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

                vk::ImageView::new(device.clone(), distance_view_create_info)
                    .expect("failed to create image view")
            })
            .collect::<Vec<_>>();

        let distance_samplers = (0..distance.len())
            .map(|_| {
                let distance_sampler_create_info = vk::SamplerCreateInfo {
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

                vk::Sampler::new(device.clone(), distance_sampler_create_info)
                    .expect("failed to create sampler")
            })
            .collect::<Vec<_>>();

        //GRAPHICS
        let mut graphics_color = (0..swapchain_images.len())
            .map(|_| {
                let graphics_color_create_info = vk::ImageCreateInfo {
                    image_type: vk::ImageType::TwoDim,
                    format: vk::Format::Rgba32Sfloat,
                    extent: (
                        render_info.extent.0 / render_info.scaling_factor,
                        render_info.extent.1 / render_info.scaling_factor,
                        1,
                    ),
                    mip_levels: 1,
                    array_layers: 1,
                    samples: vk::SAMPLE_COUNT_1,
                    tiling: vk::ImageTiling::Optimal,
                    image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT | vk::IMAGE_USAGE_STORAGE,
                    initial_layout: vk::ImageLayout::Undefined,
                };

                vk::Image::new(device.clone(), graphics_color_create_info)
                    .expect("failed to allocate image")
            })
            .collect::<Vec<_>>();

        let graphics_color_memory = graphics_color
            .iter_mut()
            .map(|graphics_color| {
                let graphics_color_memory_allocate_info = vk::MemoryAllocateInfo {
                    property_flags: vk::MEMORY_PROPERTY_DEVICE_LOCAL,
                };

                let graphics_color_memory = vk::Memory::allocate(
                    device.clone(),
                    graphics_color_memory_allocate_info,
                    graphics_color.memory_requirements(),
                    physical_device.memory_properties(),
                    false,
                )
                .expect("failed to allocate memory");

                graphics_color
                    .bind_memory(&graphics_color_memory)
                    .expect("failed to bind image to memory");

                graphics_color_memory
            })
            .collect::<Vec<_>>();

        let graphics_color_views = graphics_color
            .iter()
            .map(|graphics_color| {
                let graphics_color_view_create_info = vk::ImageViewCreateInfo {
                    image: graphics_color,
                    view_type: vk::ImageViewType::TwoDim,
                    format: vk::Format::Rgba32Sfloat,
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

                vk::ImageView::new(device.clone(), graphics_color_view_create_info)
                    .expect("failed to create image view")
            })
            .collect::<Vec<_>>();

        let graphics_color_samplers = (0..graphics_color.len())
            .map(|_| {
                let graphics_color_sampler_create_info = vk::SamplerCreateInfo {
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

                vk::Sampler::new(device.clone(), graphics_color_sampler_create_info)
                    .expect("failed to create sampler")
            })
            .collect::<Vec<_>>();

        let mut graphics_occlusion = (0..swapchain_images.len())
            .map(|_| {
                let graphics_occlusion_create_info = vk::ImageCreateInfo {
                    image_type: vk::ImageType::TwoDim,
                    format: vk::Format::Rgba32Sfloat,
                    extent: (
                        render_info.extent.0 / render_info.scaling_factor,
                        render_info.extent.1 / render_info.scaling_factor,
                        1,
                    ),
                    mip_levels: 1,
                    array_layers: 1,
                    samples: vk::SAMPLE_COUNT_1,
                    tiling: vk::ImageTiling::Optimal,
                    image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT | vk::IMAGE_USAGE_STORAGE,
                    initial_layout: vk::ImageLayout::Undefined,
                };

                vk::Image::new(device.clone(), graphics_occlusion_create_info)
                    .expect("failed to allocate image")
            })
            .collect::<Vec<_>>();

        let graphics_occlusion_memory = graphics_occlusion
            .iter_mut()
            .map(|graphics_occlusion| {
                let graphics_occlusion_memory_allocate_info = vk::MemoryAllocateInfo {
                    property_flags: vk::MEMORY_PROPERTY_DEVICE_LOCAL,
                };

                let graphics_occlusion_memory = vk::Memory::allocate(
                    device.clone(),
                    graphics_occlusion_memory_allocate_info,
                    graphics_occlusion.memory_requirements(),
                    physical_device.memory_properties(),
                    false,
                )
                .expect("failed to allocate memory");

                graphics_occlusion
                    .bind_memory(&graphics_occlusion_memory)
                    .expect("failed to bind image to memory");

                graphics_occlusion_memory
            })
            .collect::<Vec<_>>();

        let graphics_occlusion_views = graphics_occlusion
            .iter()
            .map(|graphics_occlusion| {
                let graphics_occlusion_view_create_info = vk::ImageViewCreateInfo {
                    image: graphics_occlusion,
                    view_type: vk::ImageViewType::TwoDim,
                    format: vk::Format::Rgba32Sfloat,
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

                vk::ImageView::new(device.clone(), graphics_occlusion_view_create_info)
                    .expect("failed to create image view")
            })
            .collect::<Vec<_>>();

        let graphics_occlusion_samplers = (0..graphics_occlusion.len())
            .map(|_| {
                let graphics_occlusion_sampler_create_info = vk::SamplerCreateInfo {
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

                vk::Sampler::new(device.clone(), graphics_occlusion_sampler_create_info)
                    .expect("failed to create sampler")
            })
            .collect::<Vec<_>>();

        let camera_buffer_binding = vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_VERTEX | vk::SHADER_STAGE_FRAGMENT,
        };

        let settings_buffer_binding = vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_VERTEX | vk::SHADER_STAGE_FRAGMENT,
        };

        let octree_buffer_binding = vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::StorageBuffer,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_FRAGMENT,
        };

        let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            bindings: &[
                camera_buffer_binding,
                settings_buffer_binding,
                octree_buffer_binding,
            ],
        };

        let graphics_descriptor_set_layout =
            vk::DescriptorSetLayout::new(device.clone(), descriptor_set_layout_create_info)
                .expect("failed to create descriptor set layout");

        let camera_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: swapchain_images.len() as _,
        };

        let settings_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: swapchain_images.len() as _,
        };

        let octree_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageBuffer,
            descriptor_count: swapchain_images.len() as _,
        };

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            max_sets: swapchain_images.len() as _,
            pool_sizes: &[
                camera_buffer_pool_size,
                settings_buffer_pool_size,
                octree_buffer_pool_size,
            ],
        };

        let graphics_descriptor_pool =
            vk::DescriptorPool::new(device.clone(), descriptor_pool_create_info)
                .expect("failed to create descriptor pool");

        let set_layouts = iter::repeat(&graphics_descriptor_set_layout)
            .take(swapchain_images.len() as _)
            .collect::<Vec<_>>();

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: &graphics_descriptor_pool,
            set_layouts: &set_layouts,
        };

        let graphics_descriptor_sets =
            vk::DescriptorSet::allocate(device.clone(), descriptor_set_allocate_info)
                .expect("failed to allocate descriptor sets");

        let graphics_pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            set_layouts: &[&graphics_descriptor_set_layout],
        };

        let graphics_pipeline_layout =
            vk::PipelineLayout::new(device.clone(), graphics_pipeline_layout_create_info)
                .expect("failed to create pipeline layout");

        let color_attachment_description = vk::AttachmentDescription {
            format: vk::Format::Rgba32Sfloat,
            samples: vk::SAMPLE_COUNT_1,
            load_op: vk::AttachmentLoadOp::Clear,
            store_op: vk::AttachmentStoreOp::Store,
            stencil_load_op: vk::AttachmentLoadOp::DontCare,
            stencil_store_op: vk::AttachmentStoreOp::DontCare,
            initial_layout: vk::ImageLayout::Undefined,
            final_layout: vk::ImageLayout::ColorAttachment,
        };

        let occlusion_attachment_description = vk::AttachmentDescription {
            format: vk::Format::Rgba32Sfloat,
            samples: vk::SAMPLE_COUNT_1,
            load_op: vk::AttachmentLoadOp::Clear,
            store_op: vk::AttachmentStoreOp::Store,
            stencil_load_op: vk::AttachmentLoadOp::DontCare,
            stencil_store_op: vk::AttachmentStoreOp::DontCare,
            initial_layout: vk::ImageLayout::Undefined,
            final_layout: vk::ImageLayout::ColorAttachment,
        };

        let distance_attachment_description = vk::AttachmentDescription {
            format: vk::Format::Rgba32Sfloat,
            samples: vk::SAMPLE_COUNT_1,
            load_op: vk::AttachmentLoadOp::Clear,
            store_op: vk::AttachmentStoreOp::DontCare,
            stencil_load_op: vk::AttachmentLoadOp::DontCare,
            stencil_store_op: vk::AttachmentStoreOp::DontCare,
            initial_layout: vk::ImageLayout::Undefined,
            final_layout: vk::ImageLayout::ColorAttachment,
        };

        let depth_attachment_description = vk::AttachmentDescription {
            format: vk::Format::D32Sfloat,
            samples: vk::SAMPLE_COUNT_1,
            load_op: vk::AttachmentLoadOp::Clear,
            store_op: vk::AttachmentStoreOp::Store,
            stencil_load_op: vk::AttachmentLoadOp::DontCare,
            stencil_store_op: vk::AttachmentStoreOp::DontCare,
            initial_layout: vk::ImageLayout::Undefined,
            final_layout: vk::ImageLayout::DepthStencilAttachment,
        };

        let color_attachment_reference = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::ColorAttachment,
        };

        let occlusion_attachment_reference = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::ColorAttachment,
        };

        let distance_attachment_reference = vk::AttachmentReference {
            attachment: 2,
            layout: vk::ImageLayout::ColorAttachment,
        };

        let depth_attachment_reference = vk::AttachmentReference {
            attachment: 3,
            layout: vk::ImageLayout::DepthStencilAttachment,
        };

        let subpass_description = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::Graphics,
            input_attachments: &[],
            color_attachments: &[
                color_attachment_reference,
                occlusion_attachment_reference,
                distance_attachment_reference,
            ],
            resolve_attachments: &[],
            depth_stencil_attachment: Some(&depth_attachment_reference),
            preserve_attachments: &[],
        };

        let subpass_dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT
                | vk::PIPELINE_STAGE_EARLY_FRAGMENT_TESTS,
            src_access_mask: 0,
            dst_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT
                | vk::PIPELINE_STAGE_EARLY_FRAGMENT_TESTS,
            dst_access_mask: vk::ACCESS_COLOR_ATTACHMENT_WRITE
                | vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE,
        };

        let render_pass_create_info = vk::RenderPassCreateInfo {
            attachments: &[
                color_attachment_description,
                occlusion_attachment_description,
                distance_attachment_description,
                depth_attachment_description,
            ],
            subpasses: &[subpass_description],
            dependencies: &[subpass_dependency],
        };

        let graphics_render_pass = vk::RenderPass::new(device.clone(), render_pass_create_info)
            .expect("failed to create render pass");

        let vertex_binding = vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>(),
            input_rate: vk::VertexInputRate::Vertex,
        };

        let instance_binding = vk::VertexInputBindingDescription {
            binding: 1,
            stride: mem::size_of::<Vector<u32, 3>>(),
            input_rate: vk::VertexInputRate::Instance,
        };

        let position_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::Rgb32Sfloat,
            offset: 0,
        };

        let normal_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::Rgb32Sfloat,
            offset: mem::size_of::<[f32; 3]>() as u32,
        };

        let uv_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: vk::Format::Rgb32Sfloat,
            offset: 2 * mem::size_of::<[f32; 3]>() as u32,
        };

        let chunk_position_attribute = vk::VertexInputAttributeDescription {
            binding: 1,
            location: 3,
            format: vk::Format::Rgb32Uint,
            offset: 0,
        };

        let graphics_vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            bindings: &[vertex_binding, instance_binding],
            attributes: &[
                position_attribute,
                normal_attribute,
                uv_attribute,
                chunk_position_attribute,
            ],
        };

        let graphics_pipeline = create_graphics_pipeline(
            device.clone(),
            graphics_vertex_input_info,
            graphics_shader_stages,
            &graphics_render_pass,
            &graphics_pipeline_layout,
            (render_info.extent.0 / 4, render_info.extent.1 / 4),
            3,
            vk::CULL_MODE_FRONT,
        );

        let graphics_framebuffers = graphics_color_views
            .iter()
            .zip(graphics_occlusion_views.iter())
            .zip(distance_views.iter())
            .map(
                |((graphics_color_view, graphics_occlusion_view), distance_view)| {
                    let framebuffer_create_info = vk::FramebufferCreateInfo {
                        render_pass: &graphics_render_pass,
                        attachments: &[
                            &graphics_color_view,
                            &graphics_occlusion_view,
                            &distance_view,
                            &depth_view,
                        ],
                        width: render_info.extent.0 / 4,
                        height: render_info.extent.1 / 4,
                        layers: 1,
                    };

                    vk::Framebuffer::new(device.clone(), framebuffer_create_info)
                        .expect("failed to create framebuffer")
                },
            )
            .collect::<Vec<_>>();

        //POSTFX
        let mut postfx_color = (0..swapchain_images.len())
            .map(|_| {
                let postfx_color_create_info = vk::ImageCreateInfo {
                    image_type: vk::ImageType::TwoDim,
                    format: vk::Format::Rgba32Sfloat,
                    extent: (
                        render_info.extent.0 / render_info.scaling_factor,
                        render_info.extent.1 / render_info.scaling_factor,
                        1,
                    ),
                    mip_levels: 1,
                    array_layers: 1,
                    samples: vk::SAMPLE_COUNT_1,
                    tiling: vk::ImageTiling::Optimal,
                    image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT | vk::IMAGE_USAGE_STORAGE,
                    initial_layout: vk::ImageLayout::Undefined,
                };

                vk::Image::new(device.clone(), postfx_color_create_info)
                    .expect("failed to allocate image")
            })
            .collect::<Vec<_>>();

        let postfx_color_memory = postfx_color
            .iter_mut()
            .map(|postfx_color| {
                let postfx_color_memory_allocate_info = vk::MemoryAllocateInfo {
                    property_flags: vk::MEMORY_PROPERTY_DEVICE_LOCAL,
                };

                let postfx_color_memory = vk::Memory::allocate(
                    device.clone(),
                    postfx_color_memory_allocate_info,
                    postfx_color.memory_requirements(),
                    physical_device.memory_properties(),
                    false,
                )
                .expect("failed to allocate memory");

                postfx_color
                    .bind_memory(&postfx_color_memory)
                    .expect("failed to bind image to memory");

                postfx_color_memory
            })
            .collect::<Vec<_>>();

        let postfx_color_views = postfx_color
            .iter()
            .map(|postfx_color| {
                let postfx_color_view_create_info = vk::ImageViewCreateInfo {
                    image: postfx_color,
                    view_type: vk::ImageViewType::TwoDim,
                    format: vk::Format::Rgba32Sfloat,
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

                vk::ImageView::new(device.clone(), postfx_color_view_create_info)
                    .expect("failed to create image view")
            })
            .collect::<Vec<_>>();

        let postfx_color_samplers = (0..postfx_color.len())
            .map(|_| {
                let postfx_color_sampler_create_info = vk::SamplerCreateInfo {
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

                vk::Sampler::new(device.clone(), postfx_color_sampler_create_info)
                    .expect("failed to create sampler")
            })
            .collect::<Vec<_>>();

        let settings_buffer_binding = vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_FRAGMENT,
        };

        let graphics_color_binding = vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_FRAGMENT,
        };

        let graphics_occlusion_binding = vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_FRAGMENT,
        };

        let distance_binding = vk::DescriptorSetLayoutBinding {
            binding: 3,
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_FRAGMENT,
        };

        let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            bindings: &[
                settings_buffer_binding,
                graphics_color_binding,
                graphics_occlusion_binding,
                distance_binding,
            ],
        };

        let postfx_descriptor_set_layout =
            vk::DescriptorSetLayout::new(device.clone(), descriptor_set_layout_create_info)
                .expect("failed to create descriptor set layout");

        let settings_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: swapchain_images.len() as _,
        };

        let graphics_color_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: swapchain_images.len() as _,
        };

        let graphics_occlusion_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: swapchain_images.len() as _,
        };

        let distance_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: swapchain_images.len() as _,
        };

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            max_sets: swapchain_images.len() as _,
            pool_sizes: &[
                settings_buffer_pool_size,
                graphics_color_pool_size,
                graphics_occlusion_pool_size,
                distance_pool_size,
            ],
        };

        let postfx_descriptor_pool =
            vk::DescriptorPool::new(device.clone(), descriptor_pool_create_info)
                .expect("failed to create descriptor pool");

        let set_layouts = iter::repeat(&postfx_descriptor_set_layout)
            .take(swapchain_images.len() as _)
            .collect::<Vec<_>>();

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: &postfx_descriptor_pool,
            set_layouts: &set_layouts,
        };

        let postfx_descriptor_sets =
            vk::DescriptorSet::allocate(device.clone(), descriptor_set_allocate_info)
                .expect("failed to allocate descriptor sets");

        let postfx_pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            set_layouts: &[&postfx_descriptor_set_layout],
        };

        let postfx_pipeline_layout =
            vk::PipelineLayout::new(device.clone(), postfx_pipeline_layout_create_info)
                .expect("failed to create pipeline layout");

        let color_attachment_description = vk::AttachmentDescription {
            format: vk::Format::Rgba32Sfloat,
            samples: vk::SAMPLE_COUNT_1,
            load_op: vk::AttachmentLoadOp::Clear,
            store_op: vk::AttachmentStoreOp::Store,
            stencil_load_op: vk::AttachmentLoadOp::DontCare,
            stencil_store_op: vk::AttachmentStoreOp::DontCare,
            initial_layout: vk::ImageLayout::Undefined,
            final_layout: vk::ImageLayout::ColorAttachment,
        };

        let color_attachment_reference = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::ColorAttachment,
        };

        let subpass_description = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::Graphics,
            input_attachments: &[],
            color_attachments: &[color_attachment_reference],
            resolve_attachments: &[],
            depth_stencil_attachment: None,
            preserve_attachments: &[],
        };

        let subpass_dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT
                | vk::PIPELINE_STAGE_EARLY_FRAGMENT_TESTS,
            src_access_mask: 0,
            dst_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT
                | vk::PIPELINE_STAGE_EARLY_FRAGMENT_TESTS,
            dst_access_mask: vk::ACCESS_COLOR_ATTACHMENT_WRITE
                | vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE,
        };

        let render_pass_create_info = vk::RenderPassCreateInfo {
            attachments: &[color_attachment_description],
            subpasses: &[subpass_description],
            dependencies: &[subpass_dependency],
        };

        let postfx_render_pass = vk::RenderPass::new(device.clone(), render_pass_create_info)
            .expect("failed to create render pass");

        let postfx_vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            bindings: &[],
            attributes: &[],
        };

        let postfx_pipeline = create_graphics_pipeline(
            device.clone(),
            postfx_vertex_input_info,
            postfx_shader_stages,
            &postfx_render_pass,
            &postfx_pipeline_layout,
            (render_info.extent.0 / 4, render_info.extent.1 / 4),
            1,
            vk::CULL_MODE_BACK,
        );

        let postfx_framebuffers = postfx_color_views
            .iter()
            .map(|postfx_color_view| {
                let framebuffer_create_info = vk::FramebufferCreateInfo {
                    render_pass: &postfx_render_pass,
                    attachments: &[&postfx_color_view],
                    width: render_info.extent.0 / 4,
                    height: render_info.extent.1 / 4,
                    layers: 1,
                };

                vk::Framebuffer::new(device.clone(), framebuffer_create_info)
                    .expect("failed to create framebuffer")
            })
            .collect::<Vec<_>>();

        //PRESENT
        let settings_buffer_binding = vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_FRAGMENT,
        };

        let postfx_color_binding = vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_FRAGMENT,
        };

        let look_up_table_binding = vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::CombinedImageSampler,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_FRAGMENT,
        };

        let distance_binding = vk::DescriptorSetLayoutBinding {
            binding: 3,
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: 1,
            stage: vk::SHADER_STAGE_FRAGMENT,
        };

        let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            bindings: &[
                settings_buffer_binding,
                postfx_color_binding,
                look_up_table_binding,
                distance_binding,
            ],
        };

        let present_descriptor_set_layout =
            vk::DescriptorSetLayout::new(device.clone(), descriptor_set_layout_create_info)
                .expect("failed to create descriptor set layout");

        let settings_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: swapchain_images.len() as _,
        };

        let postfx_color_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: swapchain_images.len() as _,
        };

        let look_up_table_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::CombinedImageSampler,
            descriptor_count: swapchain_images.len() as _,
        };

        let distance_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: swapchain_images.len() as _,
        };

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            max_sets: swapchain_images.len() as _,
            pool_sizes: &[
                settings_buffer_pool_size,
                postfx_color_pool_size,
                look_up_table_pool_size,
                distance_pool_size,
            ],
        };

        let present_descriptor_pool =
            vk::DescriptorPool::new(device.clone(), descriptor_pool_create_info)
                .expect("failed to create descriptor pool");

        let set_layouts = iter::repeat(&present_descriptor_set_layout)
            .take(swapchain_images.len() as _)
            .collect::<Vec<_>>();

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: &present_descriptor_pool,
            set_layouts: &set_layouts,
        };

        let present_descriptor_sets =
            vk::DescriptorSet::allocate(device.clone(), descriptor_set_allocate_info)
                .expect("failed to allocate descriptor sets");

        let present_pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            set_layouts: &[&present_descriptor_set_layout],
        };

        let present_pipeline_layout =
            vk::PipelineLayout::new(device.clone(), present_pipeline_layout_create_info)
                .expect("failed to create pipeline layout");

        let color_attachment_description = vk::AttachmentDescription {
            format: render_info.surface_format.format,
            samples: vk::SAMPLE_COUNT_1,
            load_op: vk::AttachmentLoadOp::Clear,
            store_op: vk::AttachmentStoreOp::Store,
            stencil_load_op: vk::AttachmentLoadOp::DontCare,
            stencil_store_op: vk::AttachmentStoreOp::DontCare,
            initial_layout: vk::ImageLayout::Undefined,
            final_layout: vk::ImageLayout::PresentSrc,
        };

        let depth_attachment_description = vk::AttachmentDescription {
            format: vk::Format::D32Sfloat,
            samples: vk::SAMPLE_COUNT_1,
            load_op: vk::AttachmentLoadOp::Clear,
            store_op: vk::AttachmentStoreOp::Store,
            stencil_load_op: vk::AttachmentLoadOp::DontCare,
            stencil_store_op: vk::AttachmentStoreOp::DontCare,
            initial_layout: vk::ImageLayout::Undefined,
            final_layout: vk::ImageLayout::DepthStencilAttachment,
        };

        let color_attachment_reference = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::ColorAttachment,
        };

        let depth_attachment_reference = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DepthStencilAttachment,
        };

        let subpass_description = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::Graphics,
            input_attachments: &[],
            color_attachments: &[color_attachment_reference],
            resolve_attachments: &[],
            depth_stencil_attachment: Some(&depth_attachment_reference),
            preserve_attachments: &[],
        };

        let subpass_dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: 0,
            dst_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::ACCESS_COLOR_ATTACHMENT_WRITE,
        };

        let render_pass_create_info = vk::RenderPassCreateInfo {
            attachments: &[color_attachment_description, depth_attachment_description],
            subpasses: &[subpass_description],
            dependencies: &[subpass_dependency],
        };

        let present_render_pass = vk::RenderPass::new(device.clone(), render_pass_create_info)
            .expect("failed to create render pass");

        let present_vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            bindings: &[],
            attributes: &[],
        };

        let present_pipeline = create_graphics_pipeline(
            device.clone(),
            present_vertex_input_info,
            present_shader_stages,
            &present_render_pass,
            &present_pipeline_layout,
            render_info.extent,
            1,
            vk::CULL_MODE_BACK,
        );

        let present_framebuffers = swapchain_image_views
            .iter()
            .map(|image_view| {
                let framebuffer_create_info = vk::FramebufferCreateInfo {
                    render_pass: &present_render_pass,
                    attachments: &[image_view, &depth_view],
                    width: render_info.extent.0,
                    height: render_info.extent.1,
                    layers: 1,
                };

                vk::Framebuffer::new(device.clone(), framebuffer_create_info)
                    .expect("failed to create framebuffer")
            })
            .collect::<Vec<_>>();

        Self {
            swapchain,
            swapchain_image_views,
            depth_view,
            depth_memory,
            depth_sampler,
            depth,
            distance,
            distance_memory,
            distance_views,
            distance_samplers,
            graphics_color,
            graphics_color_memory,
            graphics_color_views,
            graphics_color_samplers,
            graphics_occlusion,
            graphics_occlusion_memory,
            graphics_occlusion_views,
            graphics_occlusion_samplers,
            graphics_render_pass,
            graphics_descriptor_set_layout,
            graphics_descriptor_pool,
            graphics_descriptor_sets,
            graphics_pipeline_layout,
            graphics_pipeline,
            graphics_framebuffers,
            postfx_color,
            postfx_color_memory,
            postfx_color_views,
            postfx_color_samplers,
            postfx_render_pass,
            postfx_descriptor_set_layout,
            postfx_descriptor_pool,
            postfx_descriptor_sets,
            postfx_pipeline_layout,
            postfx_pipeline,
            postfx_framebuffers,
            present_render_pass,
            present_descriptor_set_layout,
            present_descriptor_pool,
            present_descriptor_sets,
            present_pipeline_layout,
            present_pipeline,
            present_framebuffers,
        }
    }
}

impl Vulkan {
    pub fn init(info: RendererInfo<'_>) -> Self {
        let mut octree = Octree::new();

        let ct = 2 * info.render_distance as usize * CHUNK_SIZE;
        let mut voxels = 0;

        use noise::NoiseFn;
        let perlin = noise::Perlin::new();

        for x in 0..ct {
            for z in 0..ct {
                let mut max_y = 16.0 as isize;
                for o in 1..=4 {
                    max_y += ((5.0 as f64 / (o as f64).powf(0.5))
                        * perlin.get([x as f64 / (o as f64 * 32.0), z as f64 / (o as f64 * 32.0)]))
                        as isize;
                }
                for y in 0..ct {
                    if y >= max_y as usize && y < 16 {
                        octree.place(x, y, z, 2);
                    } else if y == max_y as usize - 1 {
                        octree.place(x, y, z, 1);
                    } else if y < max_y as usize {
                        octree.place(x, y, z, 3);
                    }
                }
            }
            println!("building: {}%", ((x as f32 / ct as f32) * 100.0) as usize);
        }

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
        let debug_utils_messenger = vk::DebugUtilsMessenger::new(
            instance.clone(),
            debug_utils_messenger_create_info.unwrap(),
        )
        .expect("failed to create debug utils messenger");

        let surface = vk::Surface::new(instance.clone(), &info.window);

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

        let mut queue_family_index = None;

        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags & vk::QUEUE_GRAPHICS == 0 {
                continue;
            }
            if queue_family.queue_flags & vk::QUEUE_COMPUTE == 0 {
                continue;
            }
            if !physical_device
                .surface_supported(&surface, i as _)
                .expect("failed to query surface support")
            {
                continue;
            }
            queue_family_index = Some(i as u32);
            break;
        }

        let queue_family_index = queue_family_index.expect("failed to find suitable queue");

        let queue_create_info = vk::DeviceQueueCreateInfo {
            queue_family_index,
            queue_priorities: &[1.0],
        };

        let physical_device_features = vk::PhysicalDeviceFeatures {};

        let device_create_info = vk::DeviceCreateInfo {
            queues: &[queue_create_info],
            enabled_features: &physical_device_features,
            extensions: &[vk::KHR_SWAPCHAIN],
            layers: &layers[..],
        };

        let device = vk::Device::new(&physical_device, device_create_info)
            .expect("failed to create logical device");

        let mut queue = device.queue(queue_family_index);

        let shaders = HashMap::new();
        let shader_mod_time = HashMap::new();

        let surface_capabilities = physical_device.surface_capabilities(&surface);

        //TODO query and choose system compatible
        let surface_format = vk::SurfaceFormat {
            format: vk::Format::Bgra8Srgb,
            color_space: vk::ColorSpace::SrgbNonlinear,
        };

        //TODO query and choose system compatible
        let present_mode = vk::PresentMode::Mailbox;

        let image_count = surface_capabilities.min_image_count + 1;

        let render_info = VulkanRenderInfo {
            image_count,
            surface_format,
            surface_capabilities,
            present_mode,
            extent: (960, 540),
            scaling_factor: 4,
        };

        let render_data = None;

        let compute_data = None;

        let command_pool_create_info = vk::CommandPoolCreateInfo { queue_family_index };

        let command_pool = vk::CommandPool::new(device.clone(), command_pool_create_info)
            .expect("failed to create command pool");

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            command_pool: &command_pool,
            level: vk::CommandBufferLevel::Primary,
            count: 1,
        };

        let mut command_buffer =
            vk::CommandBuffer::allocate(device.clone(), command_buffer_allocate_info)
                .expect("failed to create command buffer")
                .remove(0);

        let semaphore_create_info = vk::SemaphoreCreateInfo {};

        let mut image_available_semaphore =
            vk::Semaphore::new(device.clone(), semaphore_create_info)
                .expect("failed to create semaphore");

        let semaphore_create_info = vk::SemaphoreCreateInfo {};

        let mut render_finished_semaphore =
            vk::Semaphore::new(device.clone(), semaphore_create_info)
                .expect("failed to create semaphore");

        let fence_create_info = vk::FenceCreateInfo {};

        let mut in_flight_fence =
            vk::Fence::new(device.clone(), fence_create_info).expect("failed to create fence");

        let last_batch = Batch::default();

        let mut instance_buffer = vk::Buffer::new(
            device.clone(),
            1048576,
            vk::BUFFER_USAGE_TRANSFER_DST | vk::BUFFER_USAGE_VERTEX,
        )
        .expect("failed to create buffer");

        let instance_buffer_memory_allocate_info = vk::MemoryAllocateInfo {
            property_flags: vk::MEMORY_PROPERTY_DEVICE_LOCAL,
        };

        let instance_buffer_memory = vk::Memory::allocate(
            device.clone(),
            instance_buffer_memory_allocate_info,
            instance_buffer.memory_requirements(),
            physical_device.memory_properties(),
            false,
        )
        .expect("failed to allocate memory");

        instance_buffer.bind_memory(&instance_buffer_memory);

        let mut data_buffer = vk::Buffer::new(
            device.clone(),
            1048576,
            vk::BUFFER_USAGE_TRANSFER_DST
                | vk::BUFFER_USAGE_VERTEX
                | vk::BUFFER_USAGE_INDEX
                | vk::BUFFER_USAGE_UNIFORM
                | vk::BUFFER_USAGE_STORAGE,
        )
        .expect("failed to create buffer");

        let data_buffer_memory_allocate_info = vk::MemoryAllocateInfo {
            property_flags: vk::MEMORY_PROPERTY_DEVICE_LOCAL,
        };

        let data_buffer_memory = vk::Memory::allocate(
            device.clone(),
            data_buffer_memory_allocate_info,
            data_buffer.memory_requirements(),
            physical_device.memory_properties(),
            false,
        )
        .expect("failed to allocate memory");

        data_buffer.bind_memory(&data_buffer_memory);

        let mut staging_buffer =
            vk::Buffer::new(device.clone(), 3000000000, vk::BUFFER_USAGE_TRANSFER_SRC)
                .expect("failed to create buffer");

        let staging_buffer_memory_allocate_info = vk::MemoryAllocateInfo {
            property_flags: vk::MEMORY_PROPERTY_HOST_VISIBLE | vk::MEMORY_PROPERTY_HOST_COHERENT,
        };

        let staging_buffer_memory = vk::Memory::allocate(
            device.clone(),
            staging_buffer_memory_allocate_info,
            staging_buffer.memory_requirements(),
            physical_device.memory_properties(),
            true,
        )
        .expect("failed to allocate memory");

        staging_buffer
            .bind_memory(&staging_buffer_memory)
            .expect("failed to bind buffer");

        let camera = Bucket::new(Camera::default());

        let mut settings = Bucket::new(RenderSettings::default());
        settings.resolution = Vector::<f32, 2>::new([960.0, 540.0]);

        let render_distance = info.render_distance;

        settings.render_distance = render_distance as u32;

        let cubelet_size = 2 * render_distance as usize * CHUNK_SIZE;

        //initial padding for octree data then octree size.
        let octree_bytes =
            2 * mem::size_of::<u32>() + octree.data().len() * mem::size_of::<crate::octree::Node>();

        let mut octree_buffer = vk::Buffer::new(
            device.clone(),
            octree_bytes as u64,
            vk::BUFFER_USAGE_TRANSFER_DST
                | vk::BUFFER_USAGE_VERTEX
                | vk::BUFFER_USAGE_INDEX
                | vk::BUFFER_USAGE_UNIFORM
                | vk::BUFFER_USAGE_STORAGE,
        )
        .expect("failed to create buffer");

        let octree_buffer_memory_allocate_info = vk::MemoryAllocateInfo {
            property_flags: vk::MEMORY_PROPERTY_DEVICE_LOCAL,
        };

        let octree_buffer_memory = vk::Memory::allocate(
            device.clone(),
            octree_buffer_memory_allocate_info,
            octree_buffer.memory_requirements(),
            physical_device.memory_properties(),
            false,
        )
        .expect("failed to allocate memory");

        octree_buffer.bind_memory(&octree_buffer_memory);

        let look_up_table_create_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TwoDim,
            format: vk::Format::Rgba8Srgb,
            extent: (256, 256, 1),
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SAMPLE_COUNT_1,
            tiling: vk::ImageTiling::Optimal,
            image_usage: vk::IMAGE_USAGE_TRANSFER_DST | vk::IMAGE_USAGE_SAMPLED,
            initial_layout: vk::ImageLayout::Undefined,
        };

        let mut look_up_table = vk::Image::new(device.clone(), look_up_table_create_info)
            .expect("failed to allocate image");

        let look_up_table_memory_allocate_info = vk::MemoryAllocateInfo {
            property_flags: vk::MEMORY_PROPERTY_DEVICE_LOCAL,
        };

        let look_up_table_memory = vk::Memory::allocate(
            device.clone(),
            look_up_table_memory_allocate_info,
            look_up_table.memory_requirements(),
            physical_device.memory_properties(),
            false,
        )
        .expect("failed to allocate memory");

        look_up_table
            .bind_memory(&look_up_table_memory)
            .expect("failed to bind memory to image");

        let look_up_table_view_create_info = vk::ImageViewCreateInfo {
            image: &look_up_table,
            view_type: vk::ImageViewType::TwoDim,
            format: vk::Format::Rgba8Srgb,
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

        let look_up_table_view = vk::ImageView::new(device.clone(), look_up_table_view_create_info)
            .expect("failed to create image view");

        let look_up_table_sampler_create_info = vk::SamplerCreateInfo {
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

        let look_up_table_sampler =
            vk::Sampler::new(device.clone(), look_up_table_sampler_create_info)
                .expect("failed to create sampler");
        /*
               let cubelet_sdf_result_create_info = vk::ImageCreateInfo {
               image_type: vk::ImageType::ThreeDim,
               format: vk::Format::R16Uint,
               extent: (cubelet_size as _, cubelet_size as _, cubelet_size as _),
               mip_levels: 1,
               array_layers: 1,
               samples: vk::SAMPLE_COUNT_1,
               tiling: vk::ImageTiling::Optimal,
               image_usage: vk::IMAGE_USAGE_TRANSFER_DST | vk::IMAGE_USAGE_STORAGE,
               initial_layout: vk::ImageLayout::Undefined,
               };

               let mut cubelet_sdf_result = vk::Image::new(device.clone(), cubelet_sdf_result_create_info)
               .expect("failed to allocate image");

               let cubelet_sdf_result_memory_allocate_info = vk::MemoryAllocateInfo {
               property_flags: vk::MEMORY_PROPERTY_DEVICE_LOCAL,
               };

               let cubelet_sdf_result_memory = vk::Memory::allocate(
               device.clone(),
               cubelet_sdf_result_memory_allocate_info,
               cubelet_sdf_result.memory_requirements(),
               physical_device.memory_properties(),
               )
               .expect("failed to allocate memory");

               cubelet_sdf_result
               .bind_memory(&cubelet_sdf_result_memory)
               .expect("failed to bind memory to image");

               let cubelet_sdf_result_view_create_info = vk::ImageViewCreateInfo {
               image: &cubelet_sdf_result,
               view_type: vk::ImageViewType::ThreeDim,
               format: vk::Format::R32Sfloat,
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

               let cubelet_sdf_result_view =
               vk::ImageView::new(device.clone(), cubelet_sdf_result_view_create_info)
               .expect("failed to create image view");

               let cubelet_sdf_result_sampler_create_info = vk::SamplerCreateInfo {
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

        let cubelet_sdf_result_sampler =
            vk::Sampler::new(device.clone(), cubelet_sdf_result_sampler_create_info)
            .expect("failed to create sampler");
        */

        staging_buffer_memory
            .write(0, |buffer_data: &'_ mut [u32]| {
                buffer_data[0] = octree.size() as _;
                buffer_data[1] = octree.data().len() as _;
            })
            .expect("failed to write to buffer");

        staging_buffer_memory
            .write(
                2 * mem::size_of::<u32>(),
                |buffer_data: &'_ mut [Node]| {
                    let octree_data = octree.data();

                    buffer_data[..octree_data.len()].copy_from_slice(&octree_data);
                },
            )
            .expect("failed to write to buffer");

        command_buffer
            .record(|commands| {
                let buffer_copy = vk::BufferCopy {
                    src_offset: 0,
                    dst_offset: 0,
                    size: octree_bytes as _,
                };

                commands.copy_buffer(&staging_buffer, &mut octree_buffer, &[buffer_copy]);
            })
            .expect("failed to record command buffer");

        let submit_info = vk::SubmitInfo {
            wait_semaphores: &[],
            wait_stages: &[],
            command_buffers: &[&command_buffer],
            signal_semaphores: &[],
        };

        queue
            .submit(&[submit_info], None)
            .expect("failed to submit buffer copy command buffer");

        queue.wait_idle().expect("failed to wait on queue");

        use image::io::Reader as ImageReader;

        let hq4x = ImageReader::open(info.hq4x)
            .expect("failed to open hq4x")
            .decode()
            .expect("failed to decode hq4x");
        let hq4x_bytes = hq4x.as_bytes();

        staging_buffer_memory
            .write(0, |data: &'_ mut [u8]| {
                data[..hq4x_bytes.len()].copy_from_slice(hq4x_bytes);
            })
            .expect("failed to write to buffer");

        command_buffer
            .record(|commands| {
                let barrier = vk::ImageMemoryBarrier {
                    old_layout: vk::ImageLayout::Undefined,
                    new_layout: vk::ImageLayout::TransferDst,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    image: &look_up_table,
                    src_access_mask: 0,
                    dst_access_mask: 0,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::IMAGE_ASPECT_COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };

                commands.pipeline_barrier(
                    vk::PIPELINE_STAGE_TOP_OF_PIPE,
                    vk::PIPELINE_STAGE_TRANSFER,
                    0,
                    &[],
                    &[],
                    &[barrier],
                );

                let buffer_image_copy = vk::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_row_length: 0,
                    buffer_image_height: 0,
                    image_subresource: vk::ImageSubresourceLayers {
                        aspect_mask: vk::IMAGE_ASPECT_COLOR,
                        mip_level: 0,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    image_offset: (0, 0, 0),
                    image_extent: (256, 256, 1),
                };

                commands.copy_buffer_to_image(
                    &staging_buffer,
                    &mut look_up_table,
                    vk::ImageLayout::TransferDst,
                    &[buffer_image_copy],
                );

                let barrier = vk::ImageMemoryBarrier {
                    old_layout: vk::ImageLayout::TransferDst,
                    new_layout: vk::ImageLayout::ShaderReadOnly,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    image: &look_up_table,
                    src_access_mask: 0,
                    dst_access_mask: 0,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::IMAGE_ASPECT_COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };

                commands.pipeline_barrier(
                    vk::PIPELINE_STAGE_TRANSFER,
                    vk::PIPELINE_STAGE_FRAGMENT_SHADER,
                    0,
                    &[],
                    &[],
                    &[barrier],
                );
            })
            .expect("failed to record command buffer");

        let submit_info = vk::SubmitInfo {
            wait_semaphores: &[],
            wait_stages: &[],
            command_buffers: &[&command_buffer],
            signal_semaphores: &[],
        };

        queue
            .submit(&[submit_info], None)
            .expect("failed to submit buffer copy command buffer");

        queue.wait_idle().expect("failed to wait on queue");

        /*
        command_buffer
        .record(|commands| {
        let cubelet_sdf_result_barrier = vk::ImageMemoryBarrier {
        old_layout: vk::ImageLayout::Undefined,
        new_layout: vk::ImageLayout::General,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: &cubelet_sdf_result,
        src_access_mask: 0,
        dst_access_mask: 0,
        subresource_range: vk::ImageSubresourceRange {
        aspect_mask: vk::IMAGE_ASPECT_COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
        },
        };

        commands.pipeline_barrier(
        vk::PIPELINE_STAGE_TOP_OF_PIPE,
        vk::PIPELINE_STAGE_COMPUTE_SHADER,
        0,
        &[],
        &[],
        &[cubelet_sdf_result_barrier],
        );
        })
        .expect("failed to record command buffer");

        let submit_info = vk::SubmitInfo {
        wait_semaphores: &[],
        wait_stages: &[],
        command_buffers: &[&command_buffer],
        signal_semaphores: &[],
        };

        queue
        .submit(&[submit_info], None)
        .expect("failed to submit buffer copy command buffer");

        queue.wait_idle().expect("failed to wait on queue");
        */

        Self {
            instance,
            #[cfg(debug_assertions)]
            debug_utils_messenger,
            surface,
            physical_device,
            device,
            queue,
            shaders,
            shader_mod_time,
            render_info,
            render_data,
            compute_data,
            command_pool,
            command_buffer,
            in_flight_fence,
            render_finished_semaphore,
            image_available_semaphore,
            last_batch,
            instance_data: vec![],
            instance_buffer,
            instance_buffer_memory,
            data_buffer,
            data_buffer_memory,
            staging_buffer,
            staging_buffer_memory,
            settings,
            last_camera: None,
            camera,
            octree,
            octree_buffer,
            octree_buffer_memory,
            look_up_table,
            look_up_table_memory,
            look_up_table_view,
            look_up_table_sampler,
            /*cubelet_sdf_result,
            cubelet_sdf_result_memory,
            cubelet_sdf_result_view,
            cubelet_sdf_result_sampler,*/
        }
    }
}

impl Renderer for Vulkan {
    fn draw_batch(&mut self, batch: Batch, entries: &'_ [Entry<'_>]) {
        let cam_pos = {
            let camera = self.camera.camera;

            let mut cam_pos = Vector::<f32, 3>::new([camera[3][0], camera[3][1], camera[3][2]]);

            let mut forwards = Vector::<f32, 4>::new([0.0, 0.0, 1.0, 0.0]);

            forwards = self.camera.view * forwards;

            //Without this, there are clipping issues.
            cam_pos += Vector::<f32, 3>::new([forwards[0], forwards[1], forwards[2]]);

            cam_pos
        };

        let last_cam_pos = {
            let camera = self.last_camera.unwrap_or_default().camera;

            let mut cam_pos = Vector::<f32, 3>::new([camera[3][0], camera[3][1], camera[3][2]]);

            let mut forwards = Vector::<f32, 4>::new([0.0, 0.0, 1.0, 0.0]);

            forwards = self.last_camera.unwrap_or_default().view * forwards;

            //Without this, there are clipping issues.
            cam_pos += Vector::<f32, 3>::new([forwards[0], forwards[1], forwards[2]]);

            cam_pos
        };

        let camera_chunk_position = Vector::<i32, 3>::new([
            (cam_pos[0] / CHUNK_SIZE as f32) as i32,
            (cam_pos[1] / CHUNK_SIZE as f32) as i32,
            (cam_pos[2] / CHUNK_SIZE as f32) as i32,
        ]);
        let last_camera_chunk_position = Vector::<i32, 3>::new([
            (last_cam_pos[0] / CHUNK_SIZE as f32) as i32,
            (last_cam_pos[1] / CHUNK_SIZE as f32) as i32,
            (last_cam_pos[2] / CHUNK_SIZE as f32) as i32,
        ]);

        let instance_offset = 65536;

        if camera_chunk_position != last_camera_chunk_position {
            let mut instance_data = HashSet::new();

            for cx in 0..2 * self.settings.render_distance as usize {
                for cy in 1..=3 {
                    for cz in 0..2 * self.settings.render_distance as usize {
                        instance_data
                            .insert(Vector::<u32, 3>::new([cx as u32, cy as u32, cz as u32]));
                    }
                }
            }

            self.instance_data = instance_data.into_iter().collect::<Vec<_>>();

            self.instance_data.sort_by(|a, b| {
                let a_pos = Vector::<f32, 3>::new([
                    a[0] as f32 * CHUNK_SIZE as f32,
                    a[1] as f32 * CHUNK_SIZE as f32,
                    a[2] as f32 * CHUNK_SIZE as f32,
                ]);

                let a_offset = Vector::<f32, 3>::new([
                    cam_pos[0]
                        .max(a_pos[0] - CHUNK_SIZE as f32 / 2.0)
                        .min(a_pos[0] + CHUNK_SIZE as f32 / 2.0),
                    cam_pos[1]
                        .max(a_pos[1] - CHUNK_SIZE as f32 / 2.0)
                        .min(a_pos[1] + CHUNK_SIZE as f32 / 2.0),
                    cam_pos[2]
                        .max(a_pos[2] - CHUNK_SIZE as f32 / 2.0)
                        .min(a_pos[2] + CHUNK_SIZE as f32 / 2.0),
                ]);

                let b_pos = Vector::<f32, 3>::new([
                    b[0] as f32 * CHUNK_SIZE as f32,
                    b[1] as f32 * CHUNK_SIZE as f32,
                    b[2] as f32 * CHUNK_SIZE as f32,
                ]);

                let b_offset = Vector::<f32, 3>::new([
                    cam_pos[0]
                        .max(b_pos[0] - CHUNK_SIZE as f32 / 2.0)
                        .min(b_pos[0] + CHUNK_SIZE as f32 / 2.0),
                    cam_pos[1]
                        .max(b_pos[1] - CHUNK_SIZE as f32 / 2.0)
                        .min(b_pos[1] + CHUNK_SIZE as f32 / 2.0),
                    cam_pos[2]
                        .max(b_pos[2] - CHUNK_SIZE as f32 / 2.0)
                        .min(b_pos[2] + CHUNK_SIZE as f32 / 2.0),
                ]);

                let a_dst = a_pos.distance(&cam_pos);

                let b_dst = b_pos.distance(&cam_pos);

                b_dst.partial_cmp(&a_dst).unwrap()
            });

            self.staging_buffer_memory
                .write(instance_offset, |data: &'_ mut [Vector<u32, 3>]| {
                    data[..self.instance_data.len()].copy_from_slice(&self.instance_data[..]);
                })
                .expect("failed to write to buffer");
        }

        self.last_camera = Some(*self.camera);

        let camera_offset = 0;

        let settings_offset = camera_offset + mem::size_of::<Camera>();
        let settings_offset = ((settings_offset as f64 / 64.0).ceil() * 64.0) as u64;

        if self.camera.is_dirty() {
            self.staging_buffer_memory
                .write(camera_offset as _, |data: &'_ mut [Camera]| {
                    data[0..1].copy_from_slice(&[*self.camera]);
                })
                .expect("failed to write to buffer");
        }

        if self.settings.is_dirty() {
            self.staging_buffer_memory
                .write(settings_offset as _, |data: &'_ mut [RenderSettings]| {
                    data[0..1].copy_from_slice(&[*self.settings]);
                })
                .expect("failed to write to buffer");
        }

        let entry_offset = settings_offset as usize + mem::size_of::<RenderSettings>();
        let entry_offset = ((entry_offset as f64 / 64.0).ceil() * 64.0) as u64;

        let mut vertex_count = 0;

        self.staging_buffer_memory
            .write(entry_offset as _, |data: &'_ mut [Vertex]| {
                for entry in entries {
                    let (vertices, _) = entry.mesh.get();

                    data[vertex_count..vertex_count + vertices.len()].copy_from_slice(&vertices);

                    vertex_count += vertices.len();
                }
            })
            .expect("failed to write to buffer");

        let mut index_count = 0;

        self.staging_buffer_memory
            .write(
                entry_offset as usize + vertex_count * mem::size_of::<Vertex>(),
                |data: &'_ mut [u16]| {
                    for entry in entries {
                        let (_, indices) = entry.mesh.get();

                        data[index_count..index_count + indices.len()].copy_from_slice(&indices);

                        index_count += indices.len();
                    }
                },
            )
            .expect("failed to write to buffer");

        //#[cfg(debug_assertions)]
        //TODO switch to shaderc
        {
            let mut base_path = std::env::current_exe().expect("failed to get current exe");
            base_path.pop();
            let base_path_str = base_path.to_str().unwrap();

            let resources_path = format!("{}/{}", base_path_str, "resources");
            let assets_path = format!("{}/{}", base_path_str, "assets");

            for entry in fs::read_dir(resources_path).expect("failed to read directory") {
                let entry = entry.expect("failed to get directory entry");

                if entry
                    .file_type()
                    .expect("failed to get file type")
                    .is_file()
                {
                    let in_path = entry.path();

                    let out_path = format!(
                        "{}/{}.spirv",
                        assets_path,
                        in_path.file_stem().unwrap().to_string_lossy(),
                    );

                    let metadata = fs::metadata(&in_path);

                    if let Err(_) = metadata {
                        continue;
                    }

                    let mod_time = metadata
                        .unwrap()
                        .modified()
                        .expect("modified on unsupported platform");

                    let last_mod_time = *self
                        .shader_mod_time
                        .entry(out_path.clone())
                        .or_insert(time::SystemTime::now());

                    if mod_time != last_mod_time {
                        if in_path.extension().and_then(|os_str| os_str.to_str()) != Some("glsl") {
                            continue;
                        }

                        let shader_type = in_path.file_stem().and_then(|stem| {
                            let stem_str = stem.to_string_lossy();

                            let stem_str_spl = stem_str.split(".").collect::<Vec<_>>();

                            let ty = stem_str_spl[stem_str_spl.len() - 1];

                            match ty {
                                "vert" => Some(glsl_to_spirv::ShaderType::Vertex),
                                "frag" => Some(glsl_to_spirv::ShaderType::Fragment),
                                "comp" => Some(glsl_to_spirv::ShaderType::Compute),
                                _ => None,
                            }
                        });

                        if let None = shader_type {
                            continue;
                        }

                        let source =
                            fs::read_to_string(&in_path).expect("failed to read shader source");

                        info!("compiling shader...");

                        let compilation_result =
                            glsl_to_spirv::compile(&source, shader_type.unwrap());

                        if let Err(e) = compilation_result {
                            error!(
                                "failed to compile shader: {}",
                                &in_path.file_stem().unwrap().to_string_lossy()
                            );
                            print!("{}", e);
                            self.shader_mod_time.insert(out_path.clone(), mod_time);
                            return;
                        }

                        let mut compilation = compilation_result.unwrap();

                        let mut compiled_bytes = vec![];

                        compilation
                            .read_to_end(&mut compiled_bytes)
                            .expect("failed to read compilation to buffer");

                        if fs::metadata(&assets_path).is_err() {
                            fs::create_dir(&assets_path)
                                .expect("failed to create assets directory");
                        }

                        if fs::metadata(&out_path).is_ok() {
                            fs::remove_file(&out_path).expect("failed to remove file");
                        }

                        fs::write(&out_path, &compiled_bytes).expect("failed to write shader");

                        self.shader_mod_time.insert(out_path.clone(), mod_time);
                        self.shaders.remove(out_path.as_str());
                    }
                }
            }
        }

        let mut reload_graphics = false;
        let mut reload_compute = false;

        self.shaders
            .entry(batch.graphics_vertex_shader.clone())
            .or_insert_with(|| {
                info!("loading vertex shader");

                reload_graphics = true;

                let bytes = fs::read(&batch.graphics_vertex_shader).unwrap();

                let code = convert_bytes_to_spirv_data(bytes);

                let shader_module_create_info = vk::ShaderModuleCreateInfo { code: &code[..] };

                let shader_module =
                    vk::ShaderModule::new(self.device.clone(), shader_module_create_info)
                        .expect("failed to create shader module");

                shader_module
            });

        self.shaders
            .entry(batch.graphics_fragment_shader.clone())
            .or_insert_with(|| {
                info!("loading fragment shader");

                reload_graphics = true;

                let bytes = fs::read(&batch.graphics_fragment_shader).unwrap();

                let code = convert_bytes_to_spirv_data(bytes);

                let shader_module_create_info = vk::ShaderModuleCreateInfo { code: &code[..] };

                let shader_module =
                    vk::ShaderModule::new(self.device.clone(), shader_module_create_info)
                        .expect("failed to create shader module");

                shader_module
            });

        self.shaders
            .entry(batch.postfx_vertex_shader.clone())
            .or_insert_with(|| {
                info!("loading vertex shader");

                reload_graphics = true;

                let bytes = fs::read(&batch.postfx_vertex_shader).unwrap();

                let code = convert_bytes_to_spirv_data(bytes);

                let shader_module_create_info = vk::ShaderModuleCreateInfo { code: &code[..] };

                let shader_module =
                    vk::ShaderModule::new(self.device.clone(), shader_module_create_info)
                        .expect("failed to create shader module");

                shader_module
            });

        self.shaders
            .entry(batch.postfx_fragment_shader.clone())
            .or_insert_with(|| {
                info!("loading fragment shader");

                reload_graphics = true;

                let bytes = fs::read(&batch.postfx_fragment_shader).unwrap();

                let code = convert_bytes_to_spirv_data(bytes);

                let shader_module_create_info = vk::ShaderModuleCreateInfo { code: &code[..] };

                let shader_module =
                    vk::ShaderModule::new(self.device.clone(), shader_module_create_info)
                        .expect("failed to create shader module");

                shader_module
            });

        self.shaders
            .entry(batch.present_vertex_shader.clone())
            .or_insert_with(|| {
                info!("loading vertex shader");

                reload_graphics = true;

                let bytes = fs::read(&batch.present_vertex_shader).unwrap();

                let code = convert_bytes_to_spirv_data(bytes);

                let shader_module_create_info = vk::ShaderModuleCreateInfo { code: &code[..] };

                let shader_module =
                    vk::ShaderModule::new(self.device.clone(), shader_module_create_info)
                        .expect("failed to create shader module");

                shader_module
            });

        self.shaders
            .entry(batch.present_fragment_shader.clone())
            .or_insert_with(|| {
                info!("loading fragment shader");

                reload_graphics = true;

                let bytes = fs::read(&batch.present_fragment_shader).unwrap();

                let code = convert_bytes_to_spirv_data(bytes);

                let shader_module_create_info = vk::ShaderModuleCreateInfo { code: &code[..] };

                let shader_module =
                    vk::ShaderModule::new(self.device.clone(), shader_module_create_info)
                        .expect("failed to create shader module");

                shader_module
            });

        self.shaders
            .entry(batch.jfa_shader.clone())
            .or_insert_with(|| {
                info!("loading jfa compute shader");

                reload_compute = true;

                let bytes = fs::read(&batch.jfa_shader).unwrap();

                let code = convert_bytes_to_spirv_data(bytes);

                let shader_module_create_info = vk::ShaderModuleCreateInfo { code: &code[..] };

                let shader_module =
                    vk::ShaderModule::new(self.device.clone(), shader_module_create_info)
                        .expect("failed to create shader module");

                shader_module
            });

        if reload_graphics
            || self.last_batch.graphics_vertex_shader != batch.graphics_vertex_shader
            || self.last_batch.graphics_fragment_shader != batch.graphics_fragment_shader
            || self.last_batch.postfx_vertex_shader != batch.postfx_vertex_shader
            || self.last_batch.postfx_fragment_shader != batch.postfx_fragment_shader
            || self.last_batch.present_vertex_shader != batch.present_vertex_shader
            || self.last_batch.present_fragment_shader != batch.present_fragment_shader
        {
            self.device.wait_idle().expect("failed to wait on device");

            let graphics_shaders = [
                vk::PipelineShaderStageCreateInfo {
                    stage: vk::SHADER_STAGE_VERTEX,
                    module: &self.shaders[&batch.graphics_vertex_shader],
                    entry_point: "main",
                },
                vk::PipelineShaderStageCreateInfo {
                    stage: vk::SHADER_STAGE_FRAGMENT,
                    module: &self.shaders[&batch.graphics_fragment_shader],
                    entry_point: "main",
                },
            ];

            let postfx_shaders = [
                vk::PipelineShaderStageCreateInfo {
                    stage: vk::SHADER_STAGE_VERTEX,
                    module: &self.shaders[&batch.postfx_vertex_shader],
                    entry_point: "main",
                },
                vk::PipelineShaderStageCreateInfo {
                    stage: vk::SHADER_STAGE_FRAGMENT,
                    module: &self.shaders[&batch.postfx_fragment_shader],
                    entry_point: "main",
                },
            ];

            let present_shaders = [
                vk::PipelineShaderStageCreateInfo {
                    stage: vk::SHADER_STAGE_VERTEX,
                    module: &self.shaders[&batch.present_vertex_shader],
                    entry_point: "main",
                },
                vk::PipelineShaderStageCreateInfo {
                    stage: vk::SHADER_STAGE_FRAGMENT,
                    module: &self.shaders[&batch.present_fragment_shader],
                    entry_point: "main",
                },
            ];

            info!("making new graphics pipeline...");

            let old_swapchain = self.render_data.take().map(|data| data.swapchain);

            self.render_data = Some(VulkanRenderData::init(
                self.device.clone(),
                &self.physical_device,
                &self.surface,
                &graphics_shaders,
                &postfx_shaders,
                &present_shaders,
                old_swapchain,
                &self.render_info,
            ));
        }

        if reload_compute || self.last_batch.jfa_shader != batch.jfa_shader {
            self.device.wait_idle().expect("failed to wait on device");

            let jfa_shader = vk::PipelineShaderStageCreateInfo {
                stage: vk::SHADER_STAGE_COMPUTE,
                module: &self.shaders[&batch.jfa_shader],
                entry_point: "main",
            };

            info!("making new compute pipelines...");

            self.compute_data = Some(VulkanComputeData::init(self.device.clone(), jfa_shader));
        }

        self.last_batch = batch;

        let render_data = self
            .render_data
            .as_mut()
            .expect("failed to retrieve render data");

        vk::Fence::wait(&[&mut self.in_flight_fence], true, u64::MAX)
            .expect("failed to wait for fence");

        vk::Fence::reset(&[&mut self.in_flight_fence]).expect("failed to reset fence");

        let image_index_result = render_data.swapchain.acquire_next_image(
            u64::MAX,
            Some(&mut self.image_available_semaphore),
            None,
        );

        let image_index = match image_index_result {
            Ok(i) => i,
            Err(e) => {
                warn!("failed to acquire next image: {:?}", e);
                return;
            }
        };

        {
            for i in 0..render_data.graphics_descriptor_sets.len() {
                let camera_buffer_info = vk::DescriptorBufferInfo {
                    buffer: &self.data_buffer,
                    offset: camera_offset as _,
                    range: mem::size_of::<Camera>(),
                };

                let camera_buffer_descriptor_write = vk::WriteDescriptorSet {
                    dst_set: &render_data.graphics_descriptor_sets[image_index as usize],
                    dst_binding: 0,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UniformBuffer,
                    buffer_infos: &[camera_buffer_info],
                    image_infos: &[],
                };

                let settings_buffer_info = vk::DescriptorBufferInfo {
                    buffer: &self.data_buffer,
                    offset: settings_offset as _,
                    range: mem::size_of::<RenderSettings>(),
                };

                let settings_buffer_descriptor_write = vk::WriteDescriptorSet {
                    dst_set: &render_data.graphics_descriptor_sets[image_index as usize],
                    dst_binding: 1,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UniformBuffer,
                    buffer_infos: &[settings_buffer_info],
                    image_infos: &[],
                };

                //initial padding for octree data then octree size.
                let octree_bytes = 2 * mem::size_of::<u32>()
                    + self.octree.data().len() * mem::size_of::<crate::octree::Node>();

                let octree_buffer_info = vk::DescriptorBufferInfo {
                    buffer: &self.octree_buffer,
                    offset: 0,
                    range: octree_bytes,
                };

                let octree_buffer_descriptor_write = vk::WriteDescriptorSet {
                    dst_set: &render_data.graphics_descriptor_sets[image_index as usize],
                    dst_binding: 2,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::StorageBuffer,
                    buffer_infos: &[octree_buffer_info],
                    image_infos: &[],
                };

                /*
                let cubelet_sdf_info = vk::DescriptorImageInfo {
                sampler: &self.cubelet_sdf_result_sampler,
                image_view: &self.cubelet_sdf_result_view,
                image_layout: vk::ImageLayout::General,
                };

                let cubelet_sdf_descriptor_write = vk::WriteDescriptorSet {
                dst_set: &render_data.graphics_descriptor_sets[image_index as usize],
                dst_binding: 2,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::StorageImage,
                buffer_infos: &[],
                image_infos: &[cubelet_sdf_info],
                };*/

                vk::DescriptorSet::update(
                    &[
                        camera_buffer_descriptor_write,
                        settings_buffer_descriptor_write,
                        octree_buffer_descriptor_write,
                        //cubelet_sdf_descriptor_write,
                    ],
                    &[],
                );
            }

            for i in 0..render_data.postfx_descriptor_sets.len() {
                let settings_buffer_info = vk::DescriptorBufferInfo {
                    buffer: &self.data_buffer,
                    offset: settings_offset as _,
                    range: mem::size_of::<RenderSettings>(),
                };

                let settings_buffer_descriptor_write = vk::WriteDescriptorSet {
                    dst_set: &render_data.postfx_descriptor_sets[image_index as usize],
                    dst_binding: 0,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UniformBuffer,
                    buffer_infos: &[settings_buffer_info],
                    image_infos: &[],
                };

                let color_info = vk::DescriptorImageInfo {
                    sampler: &render_data.graphics_color_samplers[image_index as usize],
                    image_view: &render_data.graphics_color_views[image_index as usize],
                    image_layout: vk::ImageLayout::General,
                };

                let color_descriptor_write = vk::WriteDescriptorSet {
                    dst_set: &render_data.postfx_descriptor_sets[image_index as usize],
                    dst_binding: 1,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::StorageImage,
                    buffer_infos: &[],
                    image_infos: &[color_info],
                };

                let occlusion_info = vk::DescriptorImageInfo {
                    sampler: &render_data.graphics_occlusion_samplers[image_index as usize],
                    image_view: &render_data.graphics_occlusion_views[image_index as usize],
                    image_layout: vk::ImageLayout::General,
                };

                let occlusion_descriptor_write = vk::WriteDescriptorSet {
                    dst_set: &render_data.postfx_descriptor_sets[image_index as usize],
                    dst_binding: 2,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::StorageImage,
                    buffer_infos: &[],
                    image_infos: &[occlusion_info],
                };

                let distance_info = vk::DescriptorImageInfo {
                    sampler: &render_data.distance_samplers[image_index as usize],
                    image_view: &render_data.distance_views[image_index as usize],
                    image_layout: vk::ImageLayout::General,
                };

                let distance_descriptor_write = vk::WriteDescriptorSet {
                    dst_set: &render_data.postfx_descriptor_sets[image_index as usize],
                    dst_binding: 3,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::StorageImage,
                    buffer_infos: &[],
                    image_infos: &[distance_info],
                };

                vk::DescriptorSet::update(
                    &[
                        settings_buffer_descriptor_write,
                        color_descriptor_write,
                        occlusion_descriptor_write,
                        distance_descriptor_write,
                    ],
                    &[],
                );
            }

            for i in 0..render_data.present_descriptor_sets.len() {
                let settings_buffer_info = vk::DescriptorBufferInfo {
                    buffer: &self.data_buffer,
                    offset: settings_offset as _,
                    range: mem::size_of::<RenderSettings>(),
                };

                let settings_buffer_descriptor_write = vk::WriteDescriptorSet {
                    dst_set: &render_data.present_descriptor_sets[image_index as usize],
                    dst_binding: 0,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::UniformBuffer,
                    buffer_infos: &[settings_buffer_info],
                    image_infos: &[],
                };

                let color_info = vk::DescriptorImageInfo {
                    sampler: &render_data.postfx_color_samplers[image_index as usize],
                    image_view: &render_data.postfx_color_views[image_index as usize],
                    image_layout: vk::ImageLayout::General,
                };

                let color_descriptor_write = vk::WriteDescriptorSet {
                    dst_set: &render_data.present_descriptor_sets[image_index as usize],
                    dst_binding: 1,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::StorageImage,
                    buffer_infos: &[],
                    image_infos: &[color_info],
                };

                let look_up_table_info = vk::DescriptorImageInfo {
                    sampler: &self.look_up_table_sampler,
                    image_view: &self.look_up_table_view,
                    image_layout: vk::ImageLayout::ShaderReadOnly,
                };

                let look_up_table_descriptor_write = vk::WriteDescriptorSet {
                    dst_set: &render_data.present_descriptor_sets[image_index as usize],
                    dst_binding: 2,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::CombinedImageSampler,
                    buffer_infos: &[],
                    image_infos: &[look_up_table_info],
                };

                let distance_info = vk::DescriptorImageInfo {
                    sampler: &render_data.distance_samplers[image_index as usize],
                    image_view: &render_data.distance_views[image_index as usize],
                    image_layout: vk::ImageLayout::General,
                };

                let distance_descriptor_write = vk::WriteDescriptorSet {
                    dst_set: &render_data.present_descriptor_sets[image_index as usize],
                    dst_binding: 3,
                    dst_array_element: 0,
                    descriptor_count: 1,
                    descriptor_type: vk::DescriptorType::StorageImage,
                    buffer_infos: &[],
                    image_infos: &[distance_info],
                };

                vk::DescriptorSet::update(
                    &[
                        settings_buffer_descriptor_write,
                        color_descriptor_write,
                        look_up_table_descriptor_write,
                        distance_descriptor_write,
                    ],
                    &[],
                );
            }

            self.command_buffer
                .record(|commands| {
                    if camera_chunk_position != last_camera_chunk_position {
                        let buffer_copy = vk::BufferCopy {
                            src_offset: instance_offset as _,
                            dst_offset: 0,
                            size: (self.instance_data.len() * mem::size_of::<Vector<u32, 3>>())
                                as _,
                        };

                        commands.copy_buffer(
                            &self.staging_buffer,
                            &mut self.instance_buffer,
                            &[buffer_copy],
                        );
                    }

                    if self.camera.is_dirty() {
                        let buffer_copy = vk::BufferCopy {
                            src_offset: camera_offset as _,
                            dst_offset: camera_offset as _,
                            size: mem::size_of::<Camera>() as _,
                        };

                        commands.copy_buffer(
                            &self.staging_buffer,
                            &mut self.data_buffer,
                            &[buffer_copy],
                        );
                        self.camera.clean();
                    }

                    if self.settings.is_dirty() {
                        let buffer_copy = vk::BufferCopy {
                            src_offset: settings_offset as _,
                            dst_offset: settings_offset as _,
                            size: mem::size_of::<RenderSettings>() as _,
                        };

                        commands.copy_buffer(
                            &self.staging_buffer,
                            &mut self.data_buffer,
                            &[buffer_copy],
                        );
                        self.settings.clean();
                    }

                    //Copy vertices and indices
                    let buffer_copy = vk::BufferCopy {
                        src_offset: entry_offset as _,
                        dst_offset: entry_offset as _,
                        size: (vertex_count * mem::size_of::<Vertex>()
                            + index_count * mem::size_of::<u16>())
                            as _,
                    };

                    commands.copy_buffer(
                        &self.staging_buffer,
                        &mut self.data_buffer,
                        &[buffer_copy],
                    );
                    //Graphics
                    let render_pass_begin_info = vk::RenderPassBeginInfo {
                        render_pass: &render_data.graphics_render_pass,
                        framebuffer: &render_data.graphics_framebuffers[image_index as usize],
                        render_area: vk::Rect2d {
                            offset: (0, 0),
                            extent: (
                                self.render_info.extent.0 / self.render_info.scaling_factor,
                                self.render_info.extent.1 / self.render_info.scaling_factor,
                            ),
                        },
                        color_clear_values: &[
                            [0.0385, 0.0385, 0.0385, 1.0],
                            [1.0, 1.0, 1.0, 1.0],
                            [1.0, 1.0, 1.0, 1.0],
                        ],
                        //this wont run because load_op is set to load
                        depth_stencil_clear_value: Some((1.0, 0)),
                    };

                    commands.begin_render_pass(render_pass_begin_info);

                    commands.bind_pipeline(
                        vk::PipelineBindPoint::Graphics,
                        &render_data.graphics_pipeline,
                    );

                    commands.bind_vertex_buffers(
                        0,
                        2,
                        &[&self.data_buffer, &self.instance_buffer],
                        &[entry_offset as usize, 0],
                    );

                    commands.bind_index_buffer(
                        &self.data_buffer,
                        entry_offset as usize + vertex_count * mem::size_of::<Vertex>(),
                        vk::IndexType::Uint16,
                    );

                    commands.bind_descriptor_sets(
                        vk::PipelineBindPoint::Graphics,
                        &render_data.graphics_pipeline_layout,
                        0,
                        &[&render_data.graphics_descriptor_sets[image_index as usize]],
                        &[],
                    );

                    commands.draw_indexed(index_count as _, self.instance_data.len() as _, 0, 0, 0);

                    commands.end_render_pass();

                    let color_barrier = vk::ImageMemoryBarrier {
                        old_layout: vk::ImageLayout::Undefined,
                        new_layout: vk::ImageLayout::General,
                        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        image: &render_data.graphics_color[image_index as usize],
                        src_access_mask: 0,
                        dst_access_mask: 0,
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::IMAGE_ASPECT_COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                    };

                    let occlusion_barrier = vk::ImageMemoryBarrier {
                        old_layout: vk::ImageLayout::Undefined,
                        new_layout: vk::ImageLayout::General,
                        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        image: &render_data.graphics_occlusion[image_index as usize],
                        src_access_mask: 0,
                        dst_access_mask: 0,
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::IMAGE_ASPECT_COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                    };

                    let distance_barrier = vk::ImageMemoryBarrier {
                        old_layout: vk::ImageLayout::Undefined,
                        new_layout: vk::ImageLayout::General,
                        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        image: &render_data.distance[image_index as usize],
                        src_access_mask: 0,
                        dst_access_mask: 0,
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::IMAGE_ASPECT_COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                    };

                    let render_pass_begin_info = vk::RenderPassBeginInfo {
                        render_pass: &render_data.postfx_render_pass,
                        framebuffer: &render_data.postfx_framebuffers[image_index as usize],
                        render_area: vk::Rect2d {
                            offset: (0, 0),
                            extent: (
                                self.render_info.extent.0 / self.render_info.scaling_factor,
                                self.render_info.extent.1 / self.render_info.scaling_factor,
                            ),
                        },
                        color_clear_values: &[[1.0, 0.0, 1.0, 1.0]],
                        depth_stencil_clear_value: None,
                    };

                    commands.begin_render_pass(render_pass_begin_info);

                    commands.bind_pipeline(
                        vk::PipelineBindPoint::Graphics,
                        &render_data.postfx_pipeline,
                    );

                    commands.bind_descriptor_sets(
                        vk::PipelineBindPoint::Graphics,
                        &render_data.postfx_pipeline_layout,
                        0,
                        &[&render_data.postfx_descriptor_sets[image_index as usize]],
                        &[],
                    );

                    commands.draw(3, 1, 0, 0);

                    commands.end_render_pass();

                    let color_barrier = vk::ImageMemoryBarrier {
                        old_layout: vk::ImageLayout::Undefined,
                        new_layout: vk::ImageLayout::General,
                        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        image: &render_data.postfx_color[image_index as usize],
                        src_access_mask: 0,
                        dst_access_mask: 0,
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::IMAGE_ASPECT_COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                    };

                    let render_pass_begin_info = vk::RenderPassBeginInfo {
                        render_pass: &render_data.present_render_pass,
                        framebuffer: &render_data.present_framebuffers[image_index as usize],
                        render_area: vk::Rect2d {
                            offset: (0, 0),
                            extent: self.render_info.extent,
                        },
                        color_clear_values: &[[1.0, 0.0, 1.0, 1.0]],
                        depth_stencil_clear_value: Some((1.0, 0)),
                    };

                    commands.begin_render_pass(render_pass_begin_info);

                    commands.bind_pipeline(
                        vk::PipelineBindPoint::Graphics,
                        &render_data.present_pipeline,
                    );

                    commands.bind_descriptor_sets(
                        vk::PipelineBindPoint::Graphics,
                        &render_data.present_pipeline_layout,
                        0,
                        &[&render_data.present_descriptor_sets[image_index as usize]],
                        &[],
                    );

                    commands.draw(3, 1, 0, 0);

                    commands.end_render_pass();
                })
                .expect("failed to record command buffer");

            let submit_info = vk::SubmitInfo {
                wait_semaphores: &[&self.image_available_semaphore],
                wait_stages: &[vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT],
                command_buffers: &[&self.command_buffer],
                signal_semaphores: &[&mut self.render_finished_semaphore],
            };

            self.queue
                .submit(&[submit_info], Some(&mut self.in_flight_fence))
                .expect("failed to submit draw command buffer");

            let present_info = vk::PresentInfo {
                wait_semaphores: &[&self.render_finished_semaphore],
                swapchains: &[&render_data.swapchain],
                image_indices: &[image_index],
            };

            let present_result = self.queue.present(present_info);

            match present_result {
                Ok(()) => {}
                Err(e) => warn!("failed to present: {:?}", e),
            }
        }
    }

    fn resize(&mut self, resolution: (u32, u32)) {
        self.device.wait_idle().expect("failed to wait on device");

        let graphics_shaders = [
            vk::PipelineShaderStageCreateInfo {
                stage: vk::SHADER_STAGE_VERTEX,
                module: &self.shaders[&self.last_batch.graphics_vertex_shader],
                entry_point: "main",
            },
            vk::PipelineShaderStageCreateInfo {
                stage: vk::SHADER_STAGE_FRAGMENT,
                module: &self.shaders[&self.last_batch.graphics_fragment_shader],
                entry_point: "main",
            },
        ];

        let postfx_shaders = [
            vk::PipelineShaderStageCreateInfo {
                stage: vk::SHADER_STAGE_VERTEX,
                module: &self.shaders[&self.last_batch.postfx_vertex_shader],
                entry_point: "main",
            },
            vk::PipelineShaderStageCreateInfo {
                stage: vk::SHADER_STAGE_FRAGMENT,
                module: &self.shaders[&self.last_batch.postfx_fragment_shader],
                entry_point: "main",
            },
        ];

        let present_shaders = [
            vk::PipelineShaderStageCreateInfo {
                stage: vk::SHADER_STAGE_VERTEX,
                module: &self.shaders[&self.last_batch.present_vertex_shader],
                entry_point: "main",
            },
            vk::PipelineShaderStageCreateInfo {
                stage: vk::SHADER_STAGE_FRAGMENT,
                module: &self.shaders[&self.last_batch.present_fragment_shader],
                entry_point: "main",
            },
        ];

        self.render_info.extent = resolution;
        self.settings.resolution = Vector::<f32, 2>::new([resolution.0 as _, resolution.1 as _]);

        let render_data = self.render_data.take().unwrap();

        let swapchain = render_data.swapchain;

        self.render_data = Some(VulkanRenderData::init(
            self.device.clone(),
            &self.physical_device,
            &self.surface,
            &graphics_shaders,
            &postfx_shaders,
            &present_shaders,
            Some(swapchain),
            &self.render_info,
        ));
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        self.device.wait_idle().expect("failed to wait on device");
    }
}
