use crate::bucket::Bucket;
use crate::mesh::{Mesh, Vertex};
use crate::octree::{Node, Octree, SparseOctree};
use crate::voxel::Id::*;
use crate::voxel::Voxel;

use gpu::prelude::*;
use math::prelude::{Matrix, Vector};

use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::iter;
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;
use std::time;

use log::{error, info, trace, warn};
use raw_window_handle::HasRawWindowHandle;

pub const CHUNK_SIZE: usize = 8;
static mut JFAI_DONE: bool = true;
//temporary for here for now.
#[derive(Default, Clone, Copy)]
pub struct Camera {
    pub view: Matrix<f32, 4, 4>,
    pub proj: Matrix<f32, 4, 4>,
    pub model: Matrix<f32, 4, 4>,
}

#[derive(Default, Clone, Copy)]
pub struct RenderSettings {
    pub resolution: Vector<u32, 2>,
    pub render_distance: u32,
}

pub struct RendererInfo<'a> {
    pub window: &'a dyn HasRawWindowHandle,
    pub render_distance: u32,
    pub hq4x: String,
}

pub trait Renderer {
    fn draw_objects(&mut self, batch: Batch, entries: &'_ [Object<'_>]);
    fn resize(&mut self, resolution: (u32, u32));
}

#[derive(Clone, Default)]
pub struct Batch {
    pub camera: Camera,
}

#[derive(Clone, Copy)]
pub struct Object<'a> {
    pub data: &'a SparseOctree<Voxel>,
    pub model: Matrix<f32, 4, 4>,
}

const SMALL_BUFFER: usize = 1_000_000;
const BIG_BUFFER: usize = 1_000_000_000;

const CAMERA_OFFSET: u64 = 0;
const SETTINGS_OFFSET: u64 = 256;
const VERTEX_OFFSET: u64 = 1024;
const INDEX_OFFSET: u64 = 2048;

pub struct Vulkan {
    settings: Bucket<RenderSettings>,
    last_camera: Option<Camera>,
    render_data: Option<VulkanRenderData>,
    present_fragment_shader: Shader,
    postfx_fragment_shader: Shader,
    fullscreen_vertex_shader: Shader,
    graphics_fragment_shader: Shader,
    graphics_vertex_shader: Shader,
    look_up_table_image: Image,
    octree_buffer: Buffer<[u8; BIG_BUFFER]>,
    instance_buffer: Buffer<[u8; BIG_BUFFER]>,
    data_buffer: Buffer<[u8; SMALL_BUFFER]>,
    staging_buffer: Buffer<[u8; BIG_BUFFER]>,
    device: Device,
    surface: Surface,
    context: Context,
}

pub struct VulkanRenderData {
    graphics_color: Vec<Image>,
    graphics_occlusion: Vec<Image>,
    graphics_framebuffers: Vec<Framebuffer>,
    graphics_prepass_pipeline: Pipeline,
    graphics_raycast_pipeline: Pipeline,
    graphics_render_pass: RenderPass,
    postfx_color: Vec<Image>,
    postfx_framebuffers: Vec<Framebuffer>,
    postfx_pipeline: Pipeline,
    postfx_render_pass: RenderPass,
    present_framebuffers: Vec<Framebuffer>,
    present_pipeline: Pipeline,
    present_render_pass: RenderPass,
    depth: Image,
    swapchain: Swapchain,
}

impl Vulkan {
    pub fn init(info: RendererInfo<'_>) -> Self {
        let context = Context::start();

        let surface = Surface::new(SurfaceInfo {
            context: &context,
            window: &info.window,
        });

        let mut device = Device::choose_best(DeviceInfo {
            context: &context,
            surface: &surface,
        });

        let mut staging_buffer = Buffer::<[u8; BIG_BUFFER]>::new(BufferInfo {
            device: &device,
            usage: BufferUsage::TRANSFER_SRC,
            properties: MemoryProperties::DEVICE_LOCAL,
        });

        let mut data_buffer = Buffer::<[u8; SMALL_BUFFER]>::new(BufferInfo {
            device: &device,
            usage: BufferUsage::TRANSFER_DST
                | BufferUsage::VERTEX
                | BufferUsage::INDEX
                | BufferUsage::UNIFORM,
            properties: MemoryProperties::DEVICE_LOCAL,
        });

        let instance_buffer = Buffer::<[u8; BIG_BUFFER]>::new(BufferInfo {
            device: &device,
            usage: BufferUsage::TRANSFER_DST | BufferUsage::VERTEX,
            properties: MemoryProperties::DEVICE_LOCAL,
        });

        let octree_buffer = Buffer::<[u8; BIG_BUFFER]>::new(BufferInfo {
            device: &device,
            usage: BufferUsage::TRANSFER_DST | BufferUsage::STORAGE,
            properties: MemoryProperties::DEVICE_LOCAL,
        });

        let mut look_up_table_image = Image::new(ImageInfo {
            device: &device,
            ty: ImageType::TwoDim,
            usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            format: Format::Rgba8Srgb,
            extent: (256, 256, 1),
        });

        use image::io::Reader as ImageReader;

        let hq4x = ImageReader::open(info.hq4x)
            .expect("failed to open hq4x")
            .decode()
            .expect("failed to decode hq4x");

        let hq4x_bytes = hq4x.as_bytes();

        staging_buffer.write(BufferWrite {
            offset: 0,
            data: hq4x_bytes,
        });

        device.copy_buffer_to_image(BufferImageCopy {
            from: &staging_buffer,
            to: &mut look_up_table_image,
            src: 0,
            dst_offset: (0, 0, 0),
            dst_extent: (256, 256, 1),
        });

        let mut base_path = std::env::current_exe().expect("failed to load path of executable");
        base_path.pop();
        let base_path_str = base_path.to_str().unwrap();

        let cube = format!("{}/assets/cube.obj", base_path_str);
        let cube_obj = fs::File::open(cube).expect("failed to open obj");

        let mut cube = Mesh::from_obj(cube_obj);

        let (cube_vertices, cube_indices) = cube.get();

        staging_buffer.write(BufferWrite {
            offset: VERTEX_OFFSET,
            data: &cube_vertices[..],
        });

        staging_buffer.write(BufferWrite {
            offset: INDEX_OFFSET,
            data: &cube_indices[..],
        });

        device.copy_buffer_to_buffer(BufferCopy {
            from: &staging_buffer,
            to: &mut data_buffer,
            src: VERTEX_OFFSET,
            dst: VERTEX_OFFSET,
            size: (cube_vertices.len() * mem::size_of::<Vertex>()) as u64,
        });

        device.copy_buffer_to_buffer(BufferCopy {
            from: &staging_buffer,
            to: &mut data_buffer,
            src: INDEX_OFFSET,
            dst: INDEX_OFFSET,
            size: (cube_indices.len() * mem::size_of::<u16>()) as u64,
        });

        let graphics_vertex_shader = Shader::new(ShaderInfo {
            device: &device,
            entry: "main",
            input: ShaderInput::Glsl {
                asset: PathBuf::from(format!("{}/assets/voxel.vert.spirv", base_path_str)),
                resource: PathBuf::from(format!("{}/resources/voxel.vert.glsl", base_path_str)),
            },
        });

        let graphics_fragment_shader = Shader::new(ShaderInfo {
            device: &device,
            entry: "main",
            input: ShaderInput::Glsl {
                asset: PathBuf::from(format!("{}/assets/voxel.frag.spirv", base_path_str)),
                resource: PathBuf::from(format!("{}/resources/voxel.frag.glsl", base_path_str)),
            },
        });

        let fullscreen_vertex_shader = Shader::new(ShaderInfo {
            device: &device,
            entry: "main",
            input: ShaderInput::Glsl {
                asset: PathBuf::from(format!("{}/assets/fullscreen.vert.spirv", base_path_str)),
                resource: PathBuf::from(format!(
                    "{}/resources/fullscreen.vert.glsl",
                    base_path_str
                )),
            },
        });

        let postfx_fragment_shader = Shader::new(ShaderInfo {
            device: &device,
            entry: "main",
            input: ShaderInput::Glsl {
                asset: PathBuf::from(format!("{}/assets/postfx.frag.spirv", base_path_str)),
                resource: PathBuf::from(format!("{}/resources/postfx.frag.glsl", base_path_str)),
            },
        });

        let present_fragment_shader = Shader::new(ShaderInfo {
            device: &device,
            entry: "main",
            input: ShaderInput::Glsl {
                asset: PathBuf::from(format!("{}/assets/present.frag.spirv", base_path_str)),
                resource: PathBuf::from(format!("{}/resources/present.frag.glsl", base_path_str)),
            },
        });

        let last_camera = None;

        let settings = Bucket::new(RenderSettings {
            resolution: Vector::new([960, 540]),
            render_distance: info.render_distance,
        });

        let render_data = None;

        Self {
            context,
            surface,
            device,
            staging_buffer,
            data_buffer,
            instance_buffer,
            octree_buffer,
            look_up_table_image,
            graphics_vertex_shader,
            graphics_fragment_shader,
            fullscreen_vertex_shader,
            postfx_fragment_shader,
            present_fragment_shader,
            render_data,
            settings,
            last_camera,
        }
    }
}

impl Renderer for Vulkan {
    fn draw_objects(&mut self, batch: Batch, objects: &'_ [Object<'_>]) {
        let cam_pos = {
            let mut cam_pos = batch.camera.model[3].resize();

            let mut forwards = Vector::<f32, 4>::new([0.0, 0.0, 1.0, 0.0]);

            forwards = batch.camera.view * forwards;

            //Without this, there are clipping issues.
            cam_pos += forwards.resize();

            cam_pos
        };

        let last_cam_pos = {
            let mut cam_pos = self.last_camera.unwrap_or_default().model[3].resize();

            let mut forwards = Vector::<f32, 4>::new([0.0, 0.0, 1.0, 0.0]);

            forwards = self.last_camera.unwrap_or_default().view * forwards;

            //Without this, there are clipping issues.
            cam_pos += forwards.resize();

            cam_pos
        };

        let camera_chunk_position = (cam_pos.cast() / CHUNK_SIZE as f64).castf::<i32>();

        let last_camera_chunk_position = (last_cam_pos.cast() / CHUNK_SIZE as f64).castf::<i32>();

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

            let mut instance_data = instance_data.into_iter().collect::<Vec<_>>();

            instance_data.sort_by(|&a, &b| {
                let a_pos = a.cast() * CHUNK_SIZE as f64;
                let b_pos = b.cast() * CHUNK_SIZE as f64;

                let a_dst = a_pos.distance(&cam_pos.cast());
                let b_dst = b_pos.distance(&cam_pos.cast());

                b_dst.partial_cmp(&a_dst).unwrap()
            });

            self.staging_buffer.write(BufferWrite {
                offset: 0,
                data: &instance_data[..],
            });

            self.device.copy_buffer_to_buffer(BufferCopy {
                from: &self.staging_buffer,
                to: &mut self.data_buffer,
                src: 0,
                dst: instance_offset as _,
                size: (instance_data.len() * mem::size_of::<Vector<u32, 3>>()) as u64,
            });
        }

        self.last_camera = Some(batch.camera);

        self.staging_buffer.write(BufferWrite {
            offset: CAMERA_OFFSET,
            data: &[batch.camera],
        });

        self.staging_buffer.write(BufferWrite {
            offset: SETTINGS_OFFSET,
            data: &[*self.settings],
        });

        let reload_graphics = false;

        let reload_shader = |shader: &mut Shader| match shader.reload() {
            Ok(reloaded) => {
                if reloaded {
                    reload_graphics = true;
                }
            }
            Err(err) => match err {
                ShaderError::Compilation(_, message) => {
                    error!("failed to compile shader: \n {}", message);
                }
                _ => panic!("unexpected error refreshing shader"),
            },
        };

        reload_shader(&mut self.graphics_vertex_shader);
        reload_shader(&mut self.graphics_fragment_shader);
        reload_shader(&mut self.fullscreen_vertex_shader);
        reload_shader(&mut self.postfx_fragment_shader);
        reload_shader(&mut self.present_fragment_shader);

        if reload_graphics {
            self.render_data = Some(VulkanRenderData::load(&self, self.render_data.take()));
        }

        /*vk::Fence::wait(&[&mut self.in_flight_fence], true, u64::MAX)
                .expect("failed to wait for fence");

            vk::Fence::reset(&[&mut self.in_flight_fence]).expect("failed to reset fence");
        */
        self.device.synchronize_frame();

        /*
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
        };*/

        let image_index = match self.swapchain.acquire_next_image() {
            Ok(i) => i,
            Err(e) => {
                warn!("failed to acquire next image: {:?}", e);
                return;
            }
        };

        /*
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
                        + self.octree.nodes().len() * mem::size_of::<crate::octree::Node>();

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
        */
        let octree_bytes = 2 * mem::size_of::<u32>()
            + self.octree.nodes().len() * mem::size_of::<crate::octree::Node>();

        self.render_data.graphics_prepass_pipeline.bind(
            image_index,
            &[
                Bind {
                    binding: 0,
                    count: 1,
                    ty: DescriptorType::UniformBuffer,
                    bind: BufferBind {
                        buffer: &self.data_buffer,
                        offset: CAMERA_OFFSET as _,
                        range: mem::size_of::<Camera>(),
                    },
                },
                Bind {
                    binding: 0,
                    count: 1,
                    ty: DescriptorType::UniformBuffer,
                    bind: BufferBind {
                        buffer: &self.data_buffer,
                        offset: SETTINGS_OFFSET as _,
                        range: mem::size_of::<RenderSettings>(),
                    },
                },
                Bind {
                    binding: 2,
                    count: 1,
                    ty: DescriptorType::StorageBuffer,
                    bind: BufferBind {
                        buffer: &self.octree_buffer,
                        offset: 0,
                        range: octree_bytes as _,
                    },
                },
            ],
        );

        self.render_data.graphics_raycast_pipeline.bind(
            image_index,
            &[
                Bind {
                    binding: 0,
                    count: 1,
                    ty: DescriptorType::UniformBuffer,
                    bind: BufferBind {
                        buffer: &self.data_buffer,
                        offset: CAMERA_OFFSET as _,
                        range: mem::size_of::<Camera>(),
                    },
                },
                Bind {
                    binding: 0,
                    count: 1,
                    ty: DescriptorType::UniformBuffer,
                    bind: BufferBind {
                        buffer: &self.data_buffer,
                        offset: SETTINGS_OFFSET as _,
                        range: mem::size_of::<RenderSettings>(),
                    },
                },
                Bind {
                    binding: 2,
                    count: 1,
                    ty: DescriptorType::StorageBuffer,
                    bind: BufferBind {
                        buffer: &self.octree_buffer,
                        offset: 0,
                        range: octree_bytes as _,
                    },
                },
            ],
        );

        self.render_data.postfx_pipeline.bind(
            image_index,
            &[
                Bind {
                    binding: 0,
                    count: 1,
                    ty: DescriptorType::UniformBuffer,
                    bind: BufferBind {
                        buffer: &self.data_buffer,
                        offset: SETTINGS_OFFSET as _,
                        range: mem::size_of::<RenderSettings>(),
                    },
                },
                Bind {
                    binding: 1,
                    count: 1,
                    ty: DescriptorType::StorageImage,
                    bind: ImageBind {
                        image: &self.render_data.graphics_color[image_index],
                    },
                },
                Bind {
                    binding: 2,
                    count: 1,
                    ty: DescriptorType::StorageImage,
                    bind: ImageBind {
                        image: &self.render_data.graphics_occlusion[image_index],
                    },
                },
                Bind {
                    binding: 3,
                    count: 1,
                    ty: DescriptorType::CombinedImageSampler,
                    bind: ImageBind {
                        image: &self.render_data.depth[image_index],
                    },
                },
            ],
        );

        self.render_data.present_pipeline.bind(
            image_index,
            &[
                Bind {
                    binding: 0,
                    count: 1,
                    ty: DescriptorType::UniformBuffer,
                    bind: BufferBind {
                        buffer: &self.data_buffer,
                        offset: SETTINGS_OFFSET as _,
                        range: mem::size_of::<RenderSettings>(),
                    },
                },
                Bind {
                    binding: 1,
                    count: 1,
                    ty: DescriptorType::StorageImage,
                    bind: ImageBind {
                        image: &self.render_data.postfx_color[image_index],
                    },
                },
                Bind {
                    binding: 2,
                    count: 1,
                    ty: DescriptorType::CombinedImageSampler,
                    bind: ImageBind {
                        image: &self.look_up_table,
                    },
                },
            ],
        );

        self.device
            .call(|commands| {
                let render_pass_begin_info = vk::RenderPassBeginInfo {
                    render_pass: &self.render_data.graphics_render_pass,
                    framebuffer: &self.render_data.graphics_framebuffers[image_index as usize],
                    color_clear_values: &[
                        [0.0385, 0.0385, 0.0385, 1.0],
                        [1.0, 1.0, 1.0, 1.0],
                        [1.0, 1.0, 1.0, 1.0],
                    ],
                    depth_stencil_clear_value: Some((1.0, 0)),
                };

                commands.begin_render_pass(render_pass_begin_info);

                commands.bind_pipeline(&self.render_data.graphics_pipeline);

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

                commands.draw_indexed(index_count as _, self.instance_data.len() as _, 0, 0, 0);

                commands.end_render_pass();

                commands.pipeline_barrier(&[
                    ImagePipelineBarrier {
                        old_layout: ImageLayout::Undefined,
                        new_layout: ImageLayout::General,
                        image: &self.render_data.graphics_color[image_index as usize],
                    },
                    ImagePipelineBarrier {
                        old_layout: ImageLayout::Undefined,
                        new_layout: ImageLayout::General,
                        image: &self.render_data.graphics_occlusion[image_index as usize],
                    },
                    ImagePipelineBarrier {
                        old_layout: ImageLayout::Undefined,
                        new_layout: ImageLayout::ShaderReadOnly,
                        image: &self.render_data.depth,
                    },
                ]);

                let render_pass_begin_info = vk::RenderPassBeginInfo {
                    render_pass: &self.render_data.postfx_render_pass,
                    framebuffer: &self.render_data.postfx_framebuffers[image_index as usize],
                    color_clear_values: &[[1.0, 0.0, 1.0, 1.0]],
                    depth_stencil_clear_value: None,
                };

                commands.begin_render_pass(render_pass_begin_info);

                commands.bind_pipeline(&self.render_data.postfx_pipeline);

                commands.draw(3, 1, 0, 0);

                commands.end_render_pass();

                commands.pipeline_barrier(ImagePipelineBarrier {
                    old_layout: ImageLayout::Undefined,
                    new_layout: ImageLayout::General,
                    image: &self.render_data.postfx_color[image_index as usize],
                });

                let render_pass_begin_info = vk::RenderPassBeginInfo {
                    render_pass: &render_data.present_render_pass,
                    framebuffer: &render_data.present_framebuffers[image_index as usize],
                    color_clear_values: &[[1.0, 0.0, 1.0, 1.0]],
                    depth_stencil_clear_value: Some((1.0, 0)),
                };

                commands.begin_render_pass(render_pass_begin_info);

                commands.bind_pipeline(&render_data.present_pipeline);

                commands.draw(3, 1, 0, 0);

                commands.end_render_pass();
            })
            .expect("failed to record command buffer");

        let present_result = self.device.present();

        match present_result {
            Ok(()) => {}
            Err(e) => warn!("failed to present: {:?}", e),
        }
    }

    fn resize(&mut self, resolution: (u32, u32)) {
        self.settings.resolution = Vector::<f32, 2>::new([resolution.0 as _, resolution.1 as _]);

        self.render_data = Some(VulkanRenderData::load(&self, self.render_data.take()));
    }
}

impl VulkanRenderData {
    pub fn load(vk: &Vulkan, old: Option<Self>) -> Self {
        //SWAPCHAIN
        let swapchain = Swapchain::new(SwapchainInfo {
            device: &vk.device,
            surface: &vk.surface,
            old: old.map(|old| old.swapchain),
        });

        let swapchain_images = swapchain.images();

        let present_extent = (vk.settings.resolution[0], vk.settings.resolution[1], 1);

        let graphics_extent = (
            vk.settings.resolution[0] / 4,
            vk.settings.resolution[1] / 4,
            1,
        );

        //ATTACHMENTS
        let depth = Image::new(ImageInfo {
            device: &vk.device,
            ty: ImageType::TwoDim,
            usage: ImageUsage::DEPTH_STENCIL | ImageUsage::SAMPLED,
            format: Format::D32Sfloat,
            extent: graphics_extent,
        });

        let graphics_color = (0..swapchain_images.len())
            .map(|_| {
                Image::new(ImageInfo {
                    device: &vk.device,
                    ty: ImageType::TwoDim,
                    usage: ImageUsage::COLOR | ImageUsage::STORAGE,
                    format: Format::Rgba32Sfloat,
                    extent: graphics_extent,
                })
            })
            .collect::<Vec<_>>();

        let graphics_occlusion = (0..swapchain_images.len())
            .map(|_| {
                Image::new(ImageInfo {
                    device: &vk.device,
                    ty: ImageType::TwoDim,
                    usage: ImageUsage::COLOR | ImageUsage::STORAGE,
                    format: Format::Rgba32Sfloat,
                    extent: graphics_extent,
                })
            })
            .collect::<Vec<_>>();

        let postfx_color = (0..swapchain_images.len())
            .map(|_| {
                Image::new(ImageInfo {
                    device: &vk.device,
                    ty: ImageType::TwoDim,
                    usage: ImageUsage::COLOR | ImageUsage::STORAGE,
                    format: Format::Rgba32Sfloat,
                    extent: graphics_extent,
                })
            })
            .collect::<Vec<_>>();

        //RENDERPASSES
        let graphics_render_pass = RenderPass::new(RenderPassInfo {
            device: &vk.device,
            attachments: &[
                Attachment {
                    format: Format::Rgba32Sfloat,
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::Store,
                    layout: ImageLayout::ColorAttachment,
                    ty: AttachmentType::Color,
                },
                Attachment {
                    format: Format::Rgba32Sfloat,
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::Store,
                    layout: ImageLayout::ColorAttachment,
                    ty: AttachmentType::Color,
                },
                Attachment {
                    format: Format::D32Sfloat,
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::Store,
                    layout: ImageLayout::DepthStencilAttachment,
                    ty: AttachmentType::DepthStencil,
                },
            ],
            subpasses: &[
                Subpass {
                    src: None,
                    src_access: Access::empty(),
                    src_stage: PipelineStage::EARLY_FRAGMENT_TESTS,
                    dst: Some(0),
                    dst_access: Access::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    dst_stage: PipelineStage::LATE_FRAGMENT_TESTS,
                    attachments: &[0, 1, 2],
                },
                Subpass {
                    src: Some(0),
                    src_access: Access::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    src_stage: PipelineStage::LATE_FRAGMENT_TESTS,
                    dst: Some(1),
                    dst_access: Access::COLOR_ATTACHMENT_WRITE
                        | Access::DEPTH_STENCIL_ATTACHMENT_READ,
                    dst_stage: PipelineStage::COLOR_ATTACHMENT_OUTPUT
                        | PipelineStage::EARLY_FRAGMENT_TESTS,
                    attachments: &[0, 1, 2],
                },
            ],
        });

        let postfx_render_pass = RenderPass::new(RenderPassInfo {
            device: &vk.device,
            attachments: &[Attachment {
                format: Format::Rgba32Sfloat,
                load_op: AttachmentLoadOp::Clear,
                store_op: AttachmentStoreOp::Store,
                layout: ImageLayout::ColorAttachment,
                ty: AttachmentType::Color,
            }],
            subpasses: &[Subpass {
                src: None,
                src_access: Access::empty(),
                src_stage: PipelineStage::empty(),
                dst: Some(0),
                dst_access: Access::COLOR_ATTACHMENT_WRITE,
                dst_stage: PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                attachments: &[0],
            }],
        });

        let present_render_pass = RenderPass::new(RenderPassInfo {
            device: &vk.device,
            attachments: &[Attachment {
                format: Format::Bgra8Srgb,
                load_op: AttachmentLoadOp::Clear,
                store_op: AttachmentStoreOp::Store,
                layout: ImageLayout::ColorAttachment,
                ty: AttachmentType::Color,
            }],
            subpasses: &[Subpass {
                src: None,
                src_access: Access::empty(),
                src_stage: PipelineStage::empty(),
                dst: Some(0),
                dst_access: Access::COLOR_ATTACHMENT_WRITE,
                dst_stage: PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                attachments: &[0],
            }],
        });

        //FRAMEBUFFERS
        let graphics_framebuffers = (0..swapchain_images.len())
            .map(|i| {
                Framebuffer::new(FramebufferInfo {
                    device: &vk.device,
                    render_pass: &graphics_render_pass,
                    extent: graphics_extent,
                    attachments: &[graphics_color[i], graphics_occlusion[i], depth],
                })
            })
            .collect::<Vec<_>>();

        let postfx_framebuffers = (0..swapchain_images.len())
            .map(|i| {
                Framebuffer::new(FramebufferInfo {
                    device: &vk.device,
                    render_pass: &postfx_render_pass,
                    extent: graphics_extent,
                    attachments: &[postfx_color[i]],
                })
            })
            .collect::<Vec<_>>();

        let present_framebuffers = (0..swapchain_images.len())
            .map(|i| {
                Framebuffer::new(FramebufferInfo {
                    device: &vk.device,
                    render_pass: &present_render_pass,
                    extent: present_extent,
                    attachments: &[swapchain_images[i]],
                })
            })
            .collect::<Vec<_>>();

        //PIPELINES
        let graphics_prepass_pipeline = Pipeline::new_graphics_pipeline(GraphicsPipelineInfo {
            device: &vk.device,
            render_pass: &graphics_render_pass,
            descriptor_set_count: swapchain_images.len() as _,
            color_count: 0,
            extent: (graphics_extent.0, graphics_extent.1),
            cull_mode: CullMode::FRONT,
            vertex_shader: &vk.graphics_vertex_shader,
            fragment_shader: None,
            depth_stencil: DepthStencil {
                test: true,
                write: true,
                compare_op: CompareOp::Less,
            },
            vertex_input: &[
                VertexInput {
                    binding: 0,
                    location: 0,
                    format: Format::Rgb32Sfloat,
                    rate: InputRate::Vertex,
                },
                VertexInput {
                    binding: 0,
                    location: 1,
                    format: Format::Rgb32Sfloat,
                    rate: InputRate::Vertex,
                },
                VertexInput {
                    binding: 0,
                    location: 2,
                    format: Format::Rgb32Sfloat,
                    rate: InputRate::Vertex,
                },
                VertexInput {
                    binding: 1,
                    location: 0,
                    format: Format::Rgb32Uint,
                    rate: InputRate::Instance,
                },
            ],
            layout: &[
                Descriptor {
                    binding: 0,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                    stage: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                },
                Descriptor {
                    binding: 1,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                    stage: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                },
                Descriptor {
                    binding: 2,
                    ty: DescriptorType::StorageBuffer,
                    count: 1,
                    stage: ShaderStage::FRAGMENT,
                },
            ],
        });

        let graphics_raycast_pipeline = Pipeline::new_graphics_pipeline(GraphicsPipelineInfo {
            device: &vk.device,
            render_pass: &graphics_render_pass,
            descriptor_set_count: swapchain_images.len() as _,
            color_count: 2,
            extent: (graphics_extent.0, graphics_extent.1),
            cull_mode: CullMode::FRONT,
            vertex_shader: &vk.graphics_vertex_shader,
            fragment_shader: Some(&vk.graphics_fragment_shader),
            depth_stencil: DepthStencil {
                test: true,
                write: false,
                compare_op: CompareOp::LessOrEqual,
            },
            vertex_input: &[
                VertexInput {
                    binding: 0,
                    location: 0,
                    format: Format::Rgb32Sfloat,
                    rate: InputRate::Vertex,
                },
                VertexInput {
                    binding: 0,
                    location: 1,
                    format: Format::Rgb32Sfloat,
                    rate: InputRate::Vertex,
                },
                VertexInput {
                    binding: 0,
                    location: 2,
                    format: Format::Rgb32Sfloat,
                    rate: InputRate::Vertex,
                },
                VertexInput {
                    binding: 1,
                    location: 0,
                    format: Format::Rgb32Uint,
                    rate: InputRate::Instance,
                },
            ],
            layout: &[
                Descriptor {
                    binding: 0,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                    stage: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                },
                Descriptor {
                    binding: 1,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                    stage: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                },
                Descriptor {
                    binding: 2,
                    ty: DescriptorType::StorageBuffer,
                    count: 1,
                    stage: ShaderStage::FRAGMENT,
                },
            ],
        });

        let postfx_pipeline = Pipeline::new_graphics_pipeline(GraphicsPipelineInfo {
            device: &vk.device,
            render_pass: &postfx_render_pass,
            descriptor_set_count: swapchain_images.len() as _,
            color_count: 1,
            extent: (graphics_extent.0, graphics_extent.1),
            cull_mode: CullMode::BACK,
            vertex_shader: &vk.fullscreen_vertex_shader,
            fragment_shader: Some(&vk.postfx_fragment_shader),
            depth_stencil: DepthStencil {
                test: false,
                write: false,
                compare_op: CompareOp::Always,
            },
            vertex_input: &[],
            layout: &[
                Descriptor {
                    binding: 0,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                    stage: ShaderStage::FRAGMENT,
                },
                Descriptor {
                    binding: 1,
                    ty: DescriptorType::StorageImage,
                    count: 1,
                    stage: ShaderStage::FRAGMENT,
                },
                Descriptor {
                    binding: 2,
                    ty: DescriptorType::StorageImage,
                    count: 1,
                    stage: ShaderStage::FRAGMENT,
                },
                Descriptor {
                    binding: 3,
                    ty: DescriptorType::CombinedImageSampler,
                    count: 1,
                    stage: ShaderStage::FRAGMENT,
                },
            ],
        });

        let present_pipeline = Pipeline::new_graphics_pipeline(GraphicsPipelineInfo {
            device: &vk.device,
            render_pass: &present_render_pass,
            descriptor_set_count: swapchain_images.len() as _,
            color_count: 1,
            extent: (present_extent.0, present_extent.1),
            cull_mode: CullMode::BACK,
            vertex_shader: &vk.fullscreen_vertex_shader,
            fragment_shader: Some(&vk.present_fragment_shader),
            depth_stencil: DepthStencil {
                test: false,
                write: false,
                compare_op: CompareOp::Always,
            },
            vertex_input: &[],
            layout: &[
                Descriptor {
                    binding: 0,
                    ty: DescriptorType::UniformBuffer,
                    count: 1,
                    stage: ShaderStage::FRAGMENT,
                },
                Descriptor {
                    binding: 1,
                    ty: DescriptorType::StorageImage,
                    count: 1,
                    stage: ShaderStage::FRAGMENT,
                },
                Descriptor {
                    binding: 2,
                    ty: DescriptorType::CombinedImageSampler,
                    count: 1,
                    stage: ShaderStage::FRAGMENT,
                },
            ],
        });

        Self {
            graphics_color,
            graphics_occlusion,
            graphics_framebuffers,
            graphics_render_pass,
            graphics_prepass_pipeline,
            graphics_raycast_pipeline,
            postfx_color,
            postfx_framebuffers,
            postfx_render_pass,
            postfx_pipeline,
            present_framebuffers,
            present_render_pass,
            present_pipeline,
            swapchain,
            depth,
        }
    }
}

/*
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
}
*/
/*
pub struct VulkanRenderInfo {
    swapchain_images.len(): u32,
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
            min_swapchain_images.len(): render_info.image_count,
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

        letoctree_buffer_binding = vk::DescriptorSetLayoutBinding {
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
*/

/*fn draw_batch(&mut self, batch: Batch, entries: &'_ [Entry<'_>]) {

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
}*/
