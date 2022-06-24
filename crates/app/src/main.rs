mod window;

use crate::window::{Event as WindowEvent, Window};

use std::fs;
use std::mem;

use log::{error, info, trace, warn};

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

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
    fn flush(&self) {}
}

static LOGGER: Logger = Logger;

//TODO identify why release segfaults
fn main() {
    println!("Hello, world!");

    log::set_max_level(log::LevelFilter::Info);
    log::set_logger(&LOGGER).expect("failed to set logger");

    let mut window = Window::new();

    window.rename("Octane");
    window.show();

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
    let debug_utils_messenger =
        vk::DebugUtilsMessenger::new(instance.clone(), debug_utils_messenger_create_info.unwrap())
            .expect("failed to create debug utils messenger");

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
        if queue_family.queue_flags & vk::QUEUE_GRAPHICS != 0 {
            graphics_queue_family_index = i as u32;
            break;
        }
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

    let queue = device.queue(graphics_queue_family_index);

    let surface = vk::Surface::new(instance.clone(), &window);

    let surface_capabilities = physical_device.surface_capabilities(&surface);

    //TODO query and choose system compatible
    let surface_format = vk::SurfaceFormat {
        format: vk::Format::Bgra8Srgb,
        color_space: vk::ColorSpace::SrgbNonlinear,
    };

    //TODO query and choose system compatible
    let present_mode = vk::PresentMode::Fifo;

    //TODO add dynamic size
    let extent = (1920, 1080);

    let image_count = surface_capabilities.min_image_count + 1;

    let swapchain_create_info = vk::SwapchainCreateInfo {
        surface: &surface,
        min_image_count: image_count,
        image_format: surface_format.format,
        image_color_space: surface_format.color_space,
        image_extent: extent,
        image_array_layers: 1,
        image_usage: vk::ImageUsage::ColorAttachment,
        //TODO support concurrent image sharing mode
        image_sharing_mode: vk::SharingMode::Exclusive,
        queue_family_indices: &[],
        pre_transform: surface_capabilities.current_transform,
        composite_alpha: vk::CompositeAlpha::Opaque,
        present_mode,
        clipped: true,
        old_swapchain: None,
    };

    let swapchain = vk::Swapchain::new(device.clone(), swapchain_create_info)
        .expect("failed to create swapchain");

    let swapchain_images = swapchain.images();

    let swapchain_image_views = swapchain_images
        .iter()
        .map(|image| {
            let create_info = vk::ImageViewCreateInfo {
                image,
                view_type: vk::ImageViewType::TwoDim,
                format: surface_format.format,
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

            vk::ImageView::new(device.clone(), create_info).expect("failed to create image view")
        })
        .collect::<Vec<_>>();

    let convert_bytes_to_spv_data = |bytes: Vec<u8>| {
        let endian = mem::size_of::<u32>() / mem::size_of::<u8>();
        let bits_per_byte = 8;

        if bytes.len() % endian != 0 {
            panic!("cannot convert bytes to int; too few or too many")
        }

        let mut buffer = Vec::with_capacity(bytes.len() / endian);

        for (i, byte) in bytes.into_iter().enumerate() {
            let data = byte as u32;

            if i % endian == 0 {
                buffer.push(0);
            }

            buffer[i / endian] |= data << i % endian * bits_per_byte;
        }

        buffer
    };

    let vertex_shader_code = convert_bytes_to_spv_data(
        fs::read("/home/brynn/dev/octane/assets/default.vs.spv")
            .expect("failed to read fragment shader"),
    );

    let vertex_shader_module_create_info = vk::ShaderModuleCreateInfo {
        code: &vertex_shader_code[..],
    };

    let vertex_shader_module =
        vk::ShaderModule::new(device.clone(), vertex_shader_module_create_info)
            .expect("failed to create vertex shader module");

    let fragment_shader_code = convert_bytes_to_spv_data(
        fs::read("/home/brynn/dev/octane/assets/default.fs.spv")
            .expect("failed to read fragment shader"),
    );

    let fragment_shader_module_create_info = vk::ShaderModuleCreateInfo {
        code: &fragment_shader_code[..],
    };

    let fragment_shader_module =
        vk::ShaderModule::new(device.clone(), fragment_shader_module_create_info)
            .expect("failed to create fragment shader module");

    let vertex_shader_stage_info = vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStage::Vertex,
        module: &vertex_shader_module,
        entry_point: "main",
    };

    let fragment_shader_stage_info = vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStage::Fragment,
        module: &fragment_shader_module,
        entry_point: "main",
    };

    let shader_stages = [vertex_shader_stage_info, fragment_shader_stage_info];

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {};

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TriangleList,
        primitive_restart_enable: false,
    };

    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
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
        cull_mode: vk::CULL_MODE_BACK,
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

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {};

    let pipeline_layout = vk::PipelineLayout::new(device.clone(), pipeline_layout_create_info)
        .expect("failed to create pipeline layout");

    let color_attachment_description = vk::AttachmentDescription {
        format: surface_format.format,
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

    let render_pass_create_info = vk::RenderPassCreateInfo {
        attachments: &[color_attachment_description],
        subpasses: &[subpass_description],
    };

    let render_pass = vk::RenderPass::new(device.clone(), render_pass_create_info)
        .expect("failed to create render pass");

    loop {
        let event = window.next_event();

        match event {
            Some(WindowEvent::KeyPress) | Some(WindowEvent::CloseRequested) => {
                break;
            }
            None => {}
        }
    }

    //TODO figure out surface dependency on window
    //window is dropped before surface which causes segfault
    //explicit drop fixes this but it is not ideal
    drop(swapchain);
    drop(surface);
    drop(window);
    //vk shutdown happens during implicit Drop.
    //Rc ensures shutdown happens in right order.
}
