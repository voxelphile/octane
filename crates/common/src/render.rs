use crate::mesh::Mesh;

use math::prelude::Matrix;

use std::collections::HashMap;
use std::fs;
use std::iter;
use std::mem;
use std::rc::Rc;

use log::{error, info, trace, warn};
use raw_window_handle::HasRawWindowHandle;

//temporary for here for now.
#[derive(Default, Clone, Copy)]
pub struct UniformBufferObject {
    pub model: Matrix<f32, 4, 4>,
    pub view: Matrix<f32, 4, 4>,
    pub proj: Matrix<f32, 4, 4>,
}

pub trait Renderer {
    fn draw_batch(&mut self, batch: Batch, entries: &'_ [Entry<'_>]);
    fn resize(&mut self, resolution: (u32, u32));
}

#[derive(Clone, Default)]
pub struct Batch {
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,
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

type Vertex = [f32; 3];

fn create_pipeline(
    device: Rc<vk::Device>,
    shader_stages: &'_ [vk::PipelineShaderStageCreateInfo<'_>],
    render_pass: &'_ vk::RenderPass,
    layout: &'_ vk::PipelineLayout,
    extent: (u32, u32),
) -> vk::Pipeline {
    let binding = vk::VertexInputBindingDescription {
        binding: 0,
        stride: mem::size_of::<Vertex>(),
        input_rate: vk::VertexInputRate::Vertex,
    };

    let attribute = vk::VertexInputAttributeDescription {
        binding: 0,
        location: 0,
        format: vk::Format::Rgb32Sfloat,
        offset: 0,
    };

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
        bindings: &[binding],
        attributes: &[attribute],
    };

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TriangleList,
        primitive_restart_enable: false,
    };

    let tessellation_state = vk::PipelineTessellationStateCreateInfo {};

    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: extent.0 as _,
        height: extent.1 as _,
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
        //TODO change to front and project raymarch onto backface
        cull_mode: vk::CULL_MODE_FRONT,
        front_face: vk::FrontFace::Clockwise,
        depth_bias_enable: false,
        depth_bias_constant_factor: 0.0,
        depth_bias_clamp: 0.0,
        depth_bias_slope_factor: 0.0,
        line_width: 1.0,
    };

    let multisampling = vk::PipelineMultisampleStateCreateInfo {};

    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {};

    let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
        color_write_mask: vk::COLOR_COMPONENT_R
            | vk::COLOR_COMPONENT_G
            | vk::COLOR_COMPONENT_B
            | vk::COLOR_COMPONENT_A,
        blend_enable: false,
        src_color_blend_factor: vk::BlendFactor::SrcAlpha,
        dst_color_blend_factor: vk::BlendFactor::OneMinusSrcAlpha,
        color_blend_op: vk::BlendOp::Add,
        src_alpha_blend_factor: vk::BlendFactor::One,
        dst_alpha_blend_factor: vk::BlendFactor::Zero,
        alpha_blend_op: vk::BlendOp::Add,
    };

    let color_blending = vk::PipelineColorBlendStateCreateInfo {
        logic_op_enable: false,
        logic_op: vk::LogicOp::Copy,
        attachments: &[color_blend_attachment],
        blend_constants: &[0.0, 0.0, 0.0, 0.0],
    };

    let dynamic_state = vk::PipelineDynamicStateCreateInfo {
        dynamic_states: &[],
    };

    let graphics_pipeline_create_info = vk::GraphicsPipelineCreateInfo {
        stages: &shader_stages,
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
        base_pipeline_handle: None,
        base_pipeline_index: -1,
    };

    vk::Pipeline::new_graphics_pipelines(device, None, &[graphics_pipeline_create_info])
        .expect("failed to create graphics pipeline")
        .remove(0)
}

pub struct Vulkan {
    pub ubo: UniformBufferObject,
    last_batch: Batch,
    buffer: vk::Buffer,
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
    command_buffer: vk::CommandBuffer,
    command_pool: vk::CommandPool,
    render_info: VulkanRenderInfo,
    render_data: Option<VulkanRenderData>,
    queue: vk::Queue,
    device: Rc<vk::Device>,
    shaders: HashMap<&'static str, vk::ShaderModule>,
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
}

pub struct VulkanRenderData {
    framebuffers: Vec<vk::Framebuffer>,
    graphics_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    render_pass: vk::RenderPass,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain: vk::Swapchain,
}

impl VulkanRenderData {
    pub fn init(
        device: Rc<vk::Device>,
        surface: &vk::Surface,
        shader_stages: &'_ [vk::PipelineShaderStageCreateInfo<'_>],
        old_swapchain: Option<vk::Swapchain>,
        render_info: &VulkanRenderInfo,
    ) -> Self {
        let swapchain_create_info = vk::SwapchainCreateInfo {
            surface,
            min_image_count: render_info.image_count,
            image_format: render_info.surface_format.format,
            image_color_space: render_info.surface_format.color_space,
            image_extent: render_info.extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsage::ColorAttachment,
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

        let binding = vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: 1,
            stage: vk::ShaderStage::Vertex,
        };

        let descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            bindings: &[binding],
        };

        let descriptor_set_layout =
            vk::DescriptorSetLayout::new(device.clone(), descriptor_set_layout_create_info)
                .expect("failed to create descriptor set layout");

        let descriptor_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: swapchain_images.len() as _,
        };

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            max_sets: swapchain_images.len() as _,
            pool_sizes: &[descriptor_pool_size],
        };

        let descriptor_pool = vk::DescriptorPool::new(device.clone(), descriptor_pool_create_info)
            .expect("failed to create descriptor pool");

        let set_layouts = iter::repeat(&descriptor_set_layout)
            .take(swapchain_images.len() as _)
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

        let pipeline_layout = vk::PipelineLayout::new(device.clone(), pipeline_layout_create_info)
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
            src_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: 0,
            dst_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::ACCESS_COLOR_ATTACHMENT_WRITE,
        };

        let render_pass_create_info = vk::RenderPassCreateInfo {
            attachments: &[color_attachment_description],
            subpasses: &[subpass_description],
            dependencies: &[subpass_dependency],
        };

        let render_pass = vk::RenderPass::new(device.clone(), render_pass_create_info)
            .expect("failed to create render pass");

        let framebuffers = swapchain_image_views
            .iter()
            .map(|image_view| {
                let framebuffer_create_info = vk::FramebufferCreateInfo {
                    render_pass: &render_pass,
                    attachments: &[image_view],
                    width: render_info.extent.0,
                    height: render_info.extent.1,
                    layers: 1,
                };

                vk::Framebuffer::new(device.clone(), framebuffer_create_info)
                    .expect("failed to create framebuffer")
            })
            .collect::<Vec<_>>();

        let graphics_pipeline = create_pipeline(
            device.clone(),
            shader_stages,
            &render_pass,
            &pipeline_layout,
            render_info.extent,
        );

        Self {
            swapchain,
            swapchain_image_views,
            render_pass,
            descriptor_set_layout,
            descriptor_pool,
            descriptor_sets,
            pipeline_layout,
            graphics_pipeline,
            framebuffers,
        }
    }
}

impl Vulkan {
    pub fn init(window: &'_ impl HasRawWindowHandle) -> Self {
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

        let surface = vk::Surface::new(instance.clone(), &window);

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

        let mut graphics_queue_family_index = 0;

        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags & vk::QUEUE_GRAPHICS == 0 {
                continue;
            }
            if !physical_device
                .surface_supported(&surface, i as _)
                .expect("failed to query surface support")
            {
                continue;
            }
            graphics_queue_family_index = i as u32;
            break;
        }

        let queue_create_info = vk::DeviceQueueCreateInfo {
            queue_family_index: graphics_queue_family_index,
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

        let mut queue = device.queue(graphics_queue_family_index);

        let shaders = HashMap::new();

        let surface_capabilities = physical_device.surface_capabilities(&surface);

        //TODO query and choose system compatible
        let surface_format = vk::SurfaceFormat {
            format: vk::Format::Bgra8Srgb,
            color_space: vk::ColorSpace::SrgbNonlinear,
        };

        //TODO query and choose system compatible
        let present_mode = vk::PresentMode::Fifo;

        let image_count = surface_capabilities.min_image_count + 1;

        let render_info = VulkanRenderInfo {
            image_count,
            surface_format,
            surface_capabilities,
            present_mode,
            extent: (960, 540),
        };

        let render_data = None;

        let command_pool_create_info = vk::CommandPoolCreateInfo {
            queue_family_index: graphics_queue_family_index,
        };

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

        let buffer = vk::Buffer::allocate(
            device.clone(),
            &physical_device,
            32768,
            vk::BUFFER_USAGE_VERTEX | vk::BUFFER_USAGE_INDEX | vk::BUFFER_USAGE_UNIFORM,
        )
        .expect("failed to allocate buffer");

        let ubo = UniformBufferObject::default();

        Self {
            instance,
            #[cfg(debug_assertions)]
            debug_utils_messenger,
            surface,
            device,
            queue,
            shaders,
            render_info,
            render_data,
            command_pool,
            command_buffer,
            in_flight_fence,
            render_finished_semaphore,
            image_available_semaphore,
            last_batch,
            buffer,
            ubo,
        }
    }
}

impl Renderer for Vulkan {
    fn draw_batch(&mut self, batch: Batch, entries: &'_ [Entry<'_>]) {
        self.device.wait_idle().expect("failed to wait on device");

        let mut num_vertices = 0;

        for entry in entries {
            let (vertices, _) = entry.mesh.get();

            let vertex_offset = num_vertices * mem::size_of::<Vertex>();

            self.buffer
                .copy(vertex_offset, &vertices[..])
                .expect("failed to copy to buffer");

            num_vertices += vertices.len();
        }

        let mut num_indices = 0;

        for entry in entries {
            let (_, indices) = entry.mesh.get();
            let indices = indices
                .iter()
                .map(|i| i + num_indices as u16)
                .collect::<Vec<_>>();

            let index_offset =
                num_vertices * mem::size_of::<Vertex>() + num_indices * mem::size_of::<u16>();

            self.buffer
                .copy(index_offset, &indices[..])
                .expect("failed to copy to buffer");

            num_indices += indices.len();
        }

        let ubo_offset =
            num_vertices * mem::size_of::<Vertex>() + num_indices * mem::size_of::<u16>();

        let ubo_offset = (ubo_offset as f64 / 64.0).ceil() * 64.0;

        self.buffer
            .copy(ubo_offset as _, &[self.ubo])
            .expect("failed to copy to buffer");

        self.shaders.entry(batch.vertex_shader).or_insert_with(|| {
            let bytes = fs::read(batch.vertex_shader).expect("failed to read shader");

            let code = convert_bytes_to_spirv_data(bytes);

            let shader_module_create_info = vk::ShaderModuleCreateInfo { code: &code[..] };

            let shader_module =
                vk::ShaderModule::new(self.device.clone(), shader_module_create_info)
                    .expect("failed to create shader module");

            shader_module
        });

        self.shaders
            .entry(batch.fragment_shader)
            .or_insert_with(|| {
                let bytes = fs::read(batch.fragment_shader).expect("failed to read shader");

                let code = convert_bytes_to_spirv_data(bytes);

                let shader_module_create_info = vk::ShaderModuleCreateInfo { code: &code[..] };

                let shader_module =
                    vk::ShaderModule::new(self.device.clone(), shader_module_create_info)
                        .expect("failed to create shader module");

                shader_module
            });

        if self.last_batch.vertex_shader != batch.vertex_shader
            || self.last_batch.fragment_shader != batch.fragment_shader
        {
            self.last_batch = batch;

            let shaders = [
                vk::PipelineShaderStageCreateInfo {
                    stage: vk::ShaderStage::Vertex,
                    module: &self.shaders[self.last_batch.vertex_shader],
                    entry_point: "main",
                },
                vk::PipelineShaderStageCreateInfo {
                    stage: vk::ShaderStage::Fragment,
                    module: &self.shaders[self.last_batch.fragment_shader],
                    entry_point: "main",
                },
            ];

            trace!("making new graphics pipeline...");

            self.render_data = Some(VulkanRenderData::init(
                self.device.clone(),
                &self.surface,
                &shaders,
                None,
                &self.render_info,
            ));
        }

        let render_data = self
            .render_data
            .as_mut()
            .expect("failed to retrieve render data");

        vk::Fence::wait(&[&mut self.in_flight_fence], true, u64::MAX)
            .expect("failed to wait for fence");

        vk::Fence::reset(&[&mut self.in_flight_fence]).expect("failed to reset fence");

        let image_index = render_data
            .swapchain
            .acquire_next_image(u64::MAX, Some(&mut self.image_available_semaphore), None)
            .expect("failed to acquire next image from swapchain");

        for i in 0..render_data.descriptor_sets.len() {
            let buffer_info = vk::DescriptorBufferInfo {
                buffer: &self.buffer,
                offset: ubo_offset as _,
                range: mem::size_of::<UniformBufferObject>(),
            };

            let descriptor_write = vk::WriteDescriptorSet {
                dst_set: &render_data.descriptor_sets[image_index as usize],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UniformBuffer,
                buffer_infos: &[buffer_info],
            };

            vk::DescriptorSet::update(&[descriptor_write], &[]);
        }

        self.command_buffer
            .reset()
            .expect("failed to reset command buffer");

        self.command_buffer
            .record(|commands| {
                let render_pass_begin_info = vk::RenderPassBeginInfo {
                    render_pass: &render_data.render_pass,
                    framebuffer: &render_data.framebuffers[image_index as usize],
                    render_area: vk::Rect2d {
                        offset: (0, 0),
                        extent: self.render_info.extent,
                    },
                    clear_values: &[[0.0385, 0.0385, 0.0385, 1.0]],
                };

                commands.begin_render_pass(render_pass_begin_info);

                commands.bind_pipeline(
                    vk::PipelineBindPoint::Graphics,
                    &render_data.graphics_pipeline,
                );

                commands.bind_vertex_buffers(0, 1, &[&self.buffer], &[0]);

                commands.bind_index_buffer(
                    &self.buffer,
                    num_vertices * mem::size_of::<Vertex>(),
                    vk::IndexType::Uint16,
                );

                commands.bind_descriptor_sets(
                    vk::PipelineBindPoint::Graphics,
                    &render_data.pipeline_layout,
                    0,
                    &[&render_data.descriptor_sets[image_index as usize]],
                    &[],
                );

                commands.draw_indexed(num_indices as _, 1, 0, 0, 0);

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

    fn resize(&mut self, resolution: (u32, u32)) {
        self.device.wait_idle().expect("failed to wait on device");

        let shaders = [
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStage::Vertex,
                module: &self.shaders[self.last_batch.vertex_shader],
                entry_point: "main",
            },
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStage::Fragment,
                module: &self.shaders[self.last_batch.fragment_shader],
                entry_point: "main",
            },
        ];

        self.render_info.extent = resolution;

        let render_data = self.render_data.take().unwrap();

        let swapchain = render_data.swapchain;

        self.render_data = Some(VulkanRenderData::init(
            self.device.clone(),
            &self.surface,
            &shaders,
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
