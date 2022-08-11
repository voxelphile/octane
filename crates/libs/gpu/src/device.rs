pub use crate::prelude::*;

use std::cmp;
use std::mem;
use std::rc::Rc;
use std::cell::RefCell;

use log::{error, info, trace, warn};

pub struct DeviceInfo<'a> {
    pub context: &'a Context,
    pub surface: &'a Surface,
}

#[non_exhaustive]
pub enum Device {
    Vulkan {
        instance: Rc<vk::Instance>,
        physical_device: Rc<vk::PhysicalDevice>,
        device: Rc<vk::Device>,
        queues: Vec<vk::Queue>,
        command_pool: vk::CommandPool,
        command_buffer: vk::CommandBuffer,
        descriptor_pool: vk::DescriptorPool,
        image_available_semaphore: Rc<RefCell<vk::Semaphore>>,
        render_finished_semaphore: Rc<RefCell<vk::Semaphore>>,
        in_flight_fence: vk::Fence,
    },
}

impl Device {
    pub fn choose_best(info: DeviceInfo) -> Self {
        match info.context {
            Context::Vulkan {
                instance, layers, ..
            } => {
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

                    info!("Selected GPU \"{}\"\n", properties.device_name);

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
                    if let Surface::Vulkan { surface, .. } = info.surface {
                        if !physical_device
                            .surface_supported(&surface, i as _)
                            .expect("failed to query surface support")
                        {
                            continue;
                        }
                    } else {
                        panic!("not a vulkan surface");
                    }
                    queue_family_index = Some(i as u32);
                    break;
                }

                let queue_family_index = queue_family_index.expect("failed to find suitable queue");

                let queue_create_info = vk::DeviceQueueCreateInfo {
                    queue_family_index,
                    queue_priorities: &[1.0],
                };

                let physical_device_features = vk::PhysicalDeviceFeatures {
                    shader_int_64: true,
                    ..Default::default()
                };

                let device_create_info = vk::DeviceCreateInfo {
                    queues: &[queue_create_info],
                    enabled_features: &physical_device_features,
                    extensions: &[vk::KHR_SWAPCHAIN],
                    layers: &layers[..],
                };

                let device = vk::Device::new(&physical_device, device_create_info)
                    .expect("failed to create logical device");

                let mut queue = device.queue(queue_family_index);

                let queues = vec![queue];

                let command_pool_create_info = vk::CommandPoolCreateInfo { queue_family_index };

                let command_pool = vk::CommandPool::new(device.clone(), command_pool_create_info)
                    .expect("failed to create command pool");

                let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
                    command_pool: &command_pool,
                    level: vk::CommandBufferLevel::Primary,
                    count: 1,
                };

                let command_buffer =
                    vk::CommandBuffer::allocate(device.clone(), command_buffer_allocate_info)
                        .expect("failed to create command buffer")
                        .remove(0);
        let count = 2048;

                let uniform_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: count,
        };

        let storage_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageBuffer,
            descriptor_count: count,
        };
        
        let storage_image_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: count,
        };

        let sampler_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::CombinedImageSampler,
            descriptor_count: count,
        };

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            max_sets: count,
            pool_sizes: &[
                uniform_buffer_pool_size,
                storage_buffer_pool_size,
                storage_image_pool_size,
                sampler_pool_size,
            ],
        };

        let descriptor_pool =
            vk::DescriptorPool::new(device.clone(), descriptor_pool_create_info)
                .expect("failed to create descriptor pool");

        let semaphore_create_info = vk::SemaphoreCreateInfo {};

        let image_available_semaphore =
            vk::Semaphore::new(device.clone(), semaphore_create_info)
                .expect("failed to create semaphore");

        let image_available_semaphore = Rc::new(RefCell::new(image_available_semaphore));

        let semaphore_create_info = vk::SemaphoreCreateInfo {};

        let render_finished_semaphore =
            vk::Semaphore::new(device.clone(), semaphore_create_info)
                .expect("failed to create semaphore");
        
        let render_finished_semaphore = Rc::new(RefCell::new(render_finished_semaphore));

        let fence_create_info = vk::FenceCreateInfo {};

        let in_flight_fence =
            vk::Fence::new(device.clone(), fence_create_info).expect("failed to create fence");

                Self::Vulkan {
                    instance: instance.clone(),
                    physical_device,
                    device,
                    queues,
                    command_pool,
                    command_buffer,
                    descriptor_pool,
                    image_available_semaphore,
                    render_finished_semaphore,
                    in_flight_fence,
                }
            }
        }
    }

    pub fn copy_buffer_to_buffer(&mut self, mut copy: BufferCopy<'_>) {
        match self {
            Self::Vulkan { command_buffer, queues, .. } => {
                command_buffer.record(|mut commands| {
                    let buffer_copy = vk::BufferCopy {
                        src_offset: copy.src,
                        dst_offset: copy.dst,
                        size: copy.size as _,
                    };

                    if let Buffer::Vulkan { buffer: from, .. } = &copy.from && let Buffer::Vulkan { buffer: to, .. } = &mut copy.to {
                        commands.copy_buffer(from, to, &[buffer_copy]);
                    } else {
                        panic!("not a vulkan buffer");
                    }
                }).expect("failed to record copy buffer to buffer commands");

                let submit_info = vk::SubmitInfo {
                    wait_semaphores: &[],
                    wait_stages: &[],
                    command_buffers: &[&command_buffer],
                    signal_semaphores: &[],
                };

                queues[0]
                    .submit(&[submit_info], None)
                    .expect("failed to submit buffer copy command buffer");

                queues[0].wait_idle().expect("failed to wait on queue");
            }
        }
    }

    pub fn copy_buffer_to_image(&mut self, mut copy: BufferImageCopy<'_>) {
        match self {
            Self::Vulkan { command_buffer, queues, .. } => {
                command_buffer
                    .record(|mut commands| {
                        if let Buffer::Vulkan { buffer: from, .. } = &copy.from && let Image::Vulkan { image: to, .. } = &mut copy.to { 
                            let barrier = vk::ImageMemoryBarrier {
                                old_layout: vk::ImageLayout::Undefined,
                                new_layout: vk::ImageLayout::TransferDst,
                                src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                                dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                                image: &to,
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
                                buffer_offset: copy.src as _,
                                buffer_row_length: 0,
                                buffer_image_height: 0,
                                image_subresource: vk::ImageSubresourceLayers {
                                    aspect_mask: vk::IMAGE_ASPECT_COLOR,
                                    mip_level: 0,
                                    base_array_layer: 0,
                                    layer_count: 1,
                                },
                                image_offset: copy.dst_offset,
                                image_extent: copy.dst_extent,
                            };

                            commands.copy_buffer_to_image(
                                from,
                                to,
                                vk::ImageLayout::TransferDst,
                                &[buffer_image_copy],
                            );

                            let barrier = vk::ImageMemoryBarrier {
                                old_layout: vk::ImageLayout::TransferDst,
                                new_layout: vk::ImageLayout::ShaderReadOnly,
                                src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                                dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                                image: &to,
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
                        }
                    })
                .expect("failed to record copy buffer to image commands");

                let submit_info = vk::SubmitInfo {
                    wait_semaphores: &[],
                    wait_stages: &[],
                    command_buffers: &[&command_buffer],
                    signal_semaphores: &[],
                };

                queues[0]
                    .submit(&[submit_info], None)
                    .expect("failed to submit buffer copy command buffer");

                queues[0].wait_idle().expect("failed to wait on queue");
            }
        }
    }

    pub fn synchronize(&mut self) {
        match self {
            Device::Vulkan { in_flight_fence, .. } => {
                vk::Fence::wait(&[in_flight_fence], true, u64::MAX)
                    .expect("failed to wait for fence");

                vk::Fence::reset(&[in_flight_fence]).expect("failed to reset fence");
            }
        }
    }
    
    pub fn draw_call<'a>(&'a mut self, mut script: impl FnMut(Commands<'_>)) {
        match self {
            Device::Vulkan { 
                queues,
                command_buffer,
                image_available_semaphore,
                render_finished_semaphore,
                in_flight_fence,
                .. 
            } => {
                command_buffer
                    .record(|commands| {
                        let commands = Commands::Vulkan {
                            commands
                        };

                        script(commands);
                    })
                    .expect("failed to record command buffer");

                let submit_info = vk::SubmitInfo {
                    wait_semaphores: &[&image_available_semaphore.borrow()],
                    wait_stages: &[vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT],
                    command_buffers: &[command_buffer],
                    signal_semaphores: &[&mut render_finished_semaphore.borrow_mut()],
                };

                queues[0]
                    .submit(&[submit_info], Some(in_flight_fence))
                    .expect("failed to submit draw command buffer");
            }
        }
    }

    pub fn call<'a>(&'a mut self, mut script: impl FnMut(Commands<'_>)) {
        match self {
            Device::Vulkan { 
                queues,
                command_buffer,
                .. 
            } => {
                command_buffer
                    .record(|commands| {
                        let commands = Commands::Vulkan {
                            commands
                        };

                        script(commands);
                    })
                    .expect("failed to record command buffer");

                let submit_info = vk::SubmitInfo {
                    wait_semaphores: &[],
                    wait_stages: &[],
                    command_buffers: &[command_buffer],
                    signal_semaphores: &[],
                };

                queues[0]
                    .submit(&[submit_info], None)
                    .expect("failed to submit draw command buffer");

                queues[0]
                    .wait_idle()
                    .expect("failed to wait on queue");
            }
        }
    }

    pub fn present(&mut self, swapchain: &Swapchain) -> Result<(), Error> {
        match self {
            Device::Vulkan { 
                queues,
                render_finished_semaphore,
                .. 
            } => {
                let (swapchain, &image_index) = if let Swapchain::Vulkan { swapchain, image_index, .. } = swapchain {
                    (swapchain, image_index)
                } else {
                    panic!("not a vulkan swapchain");
                };
            
                let present_info = vk::PresentInfo {
                wait_semaphores: &[&render_finished_semaphore.borrow()],
                swapchains: &[&swapchain],
                image_indices: &[image_index],
                };

                queues[0].present(present_info).map_err(|_| Error::Presentation)
            }
        }
    }
}

#[non_exhaustive]
pub enum Commands<'a> {
    Vulkan {
        commands: vk::Commands<'a>,
    }
}

impl Commands<'_> {
    pub fn begin_render_pass(&mut self, info: RenderPassBeginInfo<'_>) {
        match self {
            Self::Vulkan { commands } => {
                let RenderPass::Vulkan { render_pass, .. } = info.render_pass else { panic!("not a vulkan render pass") };
                let Framebuffer::Vulkan { framebuffer, extent, .. } = info.framebuffer else { panic!("not a vulkan framebuffer") };

                let info = vk::RenderPassBeginInfo {
                    render_pass: &render_pass,
                    framebuffer: &framebuffer,
                    render_area: vk::Rect2d {
                        offset: (0, 0),
                        extent: (extent.0, extent.1),
                    },
                    color_clear_values: &info.color_clear_values,
                    depth_stencil_clear_value: info.depth_stencil_clear_value,
                };

                commands.begin_render_pass(info);
            }
        }
    }

    pub fn end_render_pass(&mut self) {
        match self {
            Self::Vulkan { commands } => {
                commands.end_render_pass();
            }
        }
        
    }
    
    pub fn next_subpass(&mut self) {
        match self {
            Self::Vulkan { commands } => {
                commands.next_subpass();
            }
        }
        
    }
    
    pub fn bind_pipeline(&mut self, image_index: u32, pipeline: &Pipeline) {
        match self {
            Self::Vulkan { commands } => {
                let Pipeline::Vulkan { descriptor_sets, pipeline, pipeline_layout, bind_point, .. } = pipeline else { panic!("not a vulkan pipeline") };

                commands.bind_pipeline(*bind_point, pipeline);
                commands.bind_descriptor_sets(*bind_point, pipeline_layout, 0, &[&descriptor_sets[image_index as usize]], &[]);
            }
        }
        
    }
    
    pub fn bind_vertex_buffers(&mut self, 
        first_binding: u32,
        buffers: &'_ [&'_ Buffer],
        offsets: &'_ [usize],
        ) {
        match self {
            Self::Vulkan { commands } => {
                 let buffers = buffers.iter().map(|buffer| 
                     {
                        let Buffer::Vulkan { buffer, .. } = buffer else { panic!("not a vulkan buffer") };

                        buffer
                     }).collect::<Vec<_>>();

                 commands.bind_vertex_buffers(first_binding, &buffers, offsets);
            }
        }
        
    }
    
    pub fn bind_index_buffer(&mut self, buffer: &'_ Buffer, offset: usize) {
        match self {
            Self::Vulkan { commands } => {
                let Buffer::Vulkan { buffer, .. } = buffer else { panic!("not a vulkan buffer") };
                
                commands.bind_index_buffer(buffer, offset, vk::IndexType::Uint16);
            }
        }
        
    }
    
    pub fn draw(&mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
        ) {
        match self {
            Self::Vulkan { commands } => {
                commands.draw(vertex_count, instance_count, first_vertex, first_instance);
            }
        }
        
    }
    
    pub fn draw_indexed(&mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,

        ) {
        match self {
            Self::Vulkan { commands } => {
                commands.draw_indexed(index_count, instance_count, first_index, vertex_offset, first_instance);
            }
        }
        
    }

    pub fn pipeline_barrier(&mut self, src_stage: PipelineStage, dst_stage: PipelineStage, barriers: &'_ [Barrier]) {
        match self {
            Self::Vulkan { commands } => {
                let mut memory_barriers = vec![];
                let mut buffer_barriers = vec![];
                let mut image_barriers = vec![];

                for barrier in barriers {
                    match barrier {
                        Barrier::Memory {
                            src_access,
                            dst_access,
                        } => {
                            let memory_barrier = vk::MemoryBarrier {
                                src_access_mask: src_access.to_vk(),
                                dst_access_mask: dst_access.to_vk(),
                            };

                            memory_barriers.push(memory_barrier);
                        },
                        Barrier::Buffer {
                            src_access,
                            dst_access,
                            buffer,
                            offset,
                            size
                        } => {
                            let Buffer::Vulkan { buffer, .. } = buffer else { panic!("not a vulkan buffer") };
                            
                            let buffer_barrier = vk::BufferMemoryBarrier {
                                src_access_mask: src_access.to_vk(),
                                dst_access_mask: dst_access.to_vk(),
                                src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                                dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                                buffer: &buffer,
                                offset: *offset as _,
                                size: *size as _,
                           };

                            buffer_barriers.push(buffer_barrier);
                        },
                        Barrier::Image {
                            src_access,
                            dst_access,
                            old_layout,
                            new_layout,
                            image,
                        } => {
                            let Image::Vulkan { image, format, .. } = image else { panic!("not a vulkan image") };

                            let image_barrier = vk::ImageMemoryBarrier {
old_layout: old_layout.clone().into(),
                    new_layout: new_layout.clone().into(),
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    image: &image,
                    src_access_mask: src_access.to_vk(),
                    dst_access_mask: dst_access.to_vk(),
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: format.aspect_mask(),
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                            };

                            image_barriers.push(image_barrier);
                        }
                    }
                }

                commands.pipeline_barrier(
                    src_stage.to_vk(),
                    dst_stage.to_vk(),
                    0,
                    &memory_barriers,
                    &buffer_barriers,
                    &image_barriers,
                );
            }
        }
        
    }
}

pub enum Barrier<'a> {
    Memory {
        src_access: Access,
        dst_access: Access,
    },
    Buffer {
        src_access: Access,
        dst_access: Access,
        offset: usize,
        size: usize,
        buffer: &'a Buffer,
    },
    Image {
        src_access: Access,
        dst_access: Access,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
        image: &'a Image
    }
}
