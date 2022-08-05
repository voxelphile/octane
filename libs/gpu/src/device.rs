pub use crate::prelude::*;

use std::cmp;
use std::mem;
use std::rc::Rc;

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
        
                let uniform_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::UniformBuffer,
            descriptor_count: 16,
        };

        let storage_buffer_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageBuffer,
            descriptor_count: 16,
        };
        
        let storage_image_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::StorageImage,
            descriptor_count: 16,
        };

        let sampler_pool_size = vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::CombinedImageSampler,
            descriptor_count: 16,
        };

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            max_sets: 16,
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


                Self::Vulkan {
                    instance: instance.clone(),
                    physical_device,
                    device,
                    queues,
                    command_pool,
                    command_buffer,
                    descriptor_pool,
                }
            }
        }
    }

    pub fn copy_buffer_to_buffer<T, U>(&mut self, mut copy: BufferCopy<'_, T, U>) {
        match self {
            Self::Vulkan { command_buffer, queues, .. } => {
                command_buffer.record(|commands| {
                    let buffer_copy = vk::BufferCopy {
                        src_offset: copy.src,
                        dst_offset: copy.dst,
                        size: copy.size as _,
                    };

                    if let Buffer::Vulkan { buffer: from, .. } = &copy.from && let Buffer::Vulkan { buffer: to, .. } = &mut copy.to {
                        commands.copy_buffer(from, to, &[buffer_copy]);
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

    pub fn copy_buffer_to_image<T>(&mut self, mut copy: BufferImageCopy<'_, T>) {
        match self {
            Self::Vulkan { command_buffer, queues, .. } => {
                command_buffer
                    .record(|commands| {
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
}
