//TODO implement From for ffi types

use std::ffi::{CStr, CString};
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::rc::Rc;
use std::slice;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

mod ffi {
    use std::ffi::CStr;
    use std::fmt;
    use std::mem;

    macro_rules! impl_from_enum {
    ($ obj : expr, $($ name : ident => $ case : ident),*) => {
        match $obj {
            $(super::$name::$case => Self::$case),*
        }
    };
    ($ name : ident, $($cases : ident),*) => {
        impl From<super::$name> for $name {
            fn from(x: super::$name) -> Self {
                impl_from_enum!(x, $($name => $cases),*)
            }
        }
    };}

    macro_rules! impl_from_struct {
    ($ obj : expr, $($ case : ident),* { $($Self:tt)* }) => {
        $($Self)* {
            $($case: $obj.$case as _),*
        }
    };
    ($ name : ident, $($cases : ident),*) => {
        impl From<super::$name> for $name {
            fn from(x: super::$name) -> Self {
                impl_from_struct!(x, $($cases),* { Self })
            }
        }
    };}

    macro_rules! handle {
        ($ name : ident) => {
            #[repr(transparent)]
            #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
            pub struct $name(*mut u8);

            impl Default for $name {
                fn default() -> Self {
                    Self::null()
                }
            }

            unsafe impl Send for $name {}
            unsafe impl Sync for $name {}

            impl $name {
                pub const fn null() -> Self {
                    Self(::std::ptr::null_mut())
                }
            }

            impl fmt::Pointer for $name {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    ::std::fmt::Pointer::fmt(&self.0, f)
                }
            }

            impl fmt::Debug for $name {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    ::std::fmt::Debug::fmt(&self.0, f)
                }
            }
        };
    }

    macro_rules! handle_nondispatchable {
        ($ name : ident) => {
            #[repr(transparent)]
            #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
            pub struct $name(u64);

            impl Default for $name {
                fn default() -> Self {
                    Self::null()
                }
            }

            impl $name {
                pub const fn null() -> Self {
                    Self(0)
                }
            }

            impl ::std::fmt::Pointer for $name {
                fn fmt(&self, f: &mut fmt::Formatter) -> ::std::fmt::Result {
                    write!(f, "0x{:x}", self.0)
                }
            }

            impl ::std::fmt::Debug for $name {
                fn fmt(&self, f: &mut fmt::Formatter) -> ::std::fmt::Result {
                    write!(f, "0x{:x}", self.0)
                }
            }
        };
    }

    handle!(Instance);
    handle!(PhysicalDevice);
    handle!(Device);
    handle!(Queue);
    handle!(CommandBuffer);

    handle_nondispatchable!(DebugUtilsMessenger);
    handle_nondispatchable!(Surface);
    handle_nondispatchable!(Swapchain);
    handle_nondispatchable!(Image);
    handle_nondispatchable!(ImageView);
    handle_nondispatchable!(ShaderModule);
    handle_nondispatchable!(DescriptorSetLayout);
    handle_nondispatchable!(PipelineLayout);
    handle_nondispatchable!(RenderPass);
    handle_nondispatchable!(PipelineCache);
    handle_nondispatchable!(Pipeline);
    handle_nondispatchable!(Framebuffer);
    handle_nondispatchable!(CommandPool);
    handle_nondispatchable!(Fence);
    handle_nondispatchable!(Semaphore);
    handle_nondispatchable!(Buffer);
    handle_nondispatchable!(DeviceMemory);
    handle_nondispatchable!(DescriptorPool);
    handle_nondispatchable!(DescriptorSet);
    handle_nondispatchable!(Sampler);

    pub type DeviceSize = u64;
    pub type Flags = u32;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub enum Result {
        Success = 0,
        NotReady = 1,
        Timeout = 2,
        EventSet = 3,
        EventReset = 4,
        Incomplete = 5,
        OutOfHostMemory = -1,
        OutOfDeviceMemory = -2,
        InitializationFailed = -3,
        DeviceLost = -4,
        MemoryMapFailed = -5,
        LayerNotPresent = -6,
        ExtensionNotPresent = -7,
        FeatureNotPresent = -8,
        IncompatibleDriver = -9,
        TooManyObjects = -10,
        FormatNotSupported = -11,
        FragmentedPool = -12,
        Unknown = -13,
        OutOfPoolMemory = -1000069000,
        Fragmentation = -1000161000,
        InvalidExternalHandle = -1000072003,
        SurfaceLost = -1000000000,
        NativeWindowInUse = -1000000001,
        ValidationFailed = -1000011001,
        Suboptimal = 1000001003,
        OutOfDate = -1000001004,
        InvalidShader = -1000012000,
        FullScreenExclusiveModeLost = -1000255000,
        InvalidOpaqueCaptureAddress = -1000257000,
        CompressionExhausted = -1000338000,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum StructureType {
        ApplicationInfo = 0,
        InstanceCreateInfo = 1,
        DeviceQueueCreateInfo = 2,
        DeviceCreateInfo = 3,
        SubmitInfo = 4,
        MemoryAllocateInfo = 5,
        FenceCreateInfo = 8,
        SemaphoreCreateInfo = 9,
        BufferCreateInfo = 12,
        ImageCreateInfo = 14,
        ImageViewCreateInfo = 15,
        ShaderModuleCreateInfo = 16,
        PipelineShaderStageCreateInfo = 18,
        PipelineVertexInputStateCreateInfo = 19,
        PipelineInputAssemblyStateCreateInfo = 20,
        PipelineTessellationStateCreateInfo = 21,
        PipelineViewportStateCreateInfo = 22,
        PipelineRasterizationStateCreateInfo = 23,
        PipelineMultisampleStateCreateInfo = 24,
        PipelineDepthStencilStateCreateInfo = 25,
        PipelineColorBlendStateCreateInfo = 26,
        PipelineDynamicStateCreateInfo = 27,
        GraphicsPipelineCreateInfo = 28,
        ComputePipelineCreateInfo = 29,
        PipelineLayoutCreateInfo = 30,
        SamplerCreateInfo = 31,
        DescriptorSetLayoutCreateInfo = 32,
        DescriptorPoolCreateInfo = 33,
        DescriptorSetAllocateInfo = 34,
        WriteDescriptorSet = 35,
        CopyDescriptorSet = 36,
        FramebufferCreateInfo = 37,
        RenderPassCreateInfo = 38,
        CommandPoolCreateInfo = 39,
        CommandBufferAllocateInfo = 40,
        CommandBufferBeginInfo = 42,
        RenderPassBeginInfo = 43,
        BufferMemoryBarrier = 44,
        ImageMemoryBarrier = 45,
        MemoryBarrier = 46,
        SwapchainCreateInfo = 1000001000,
        PresentInfo = 1000001001,
        XlibSurfaceCreateInfo = 1000004000,
        Win32SurfaceCreateInfo = 1000009000,
        DebugUtilsMessengerCreateInfo = 1000128004,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum ObjectType {
        Unknown = 0,
        Instance = 1,
        PhysicalDevice = 2,
        Device = 3,
        Queue = 4,
        Semaphore = 5,
        CommandBuffer = 6,
        Fence = 7,
        DeviceMemory = 8,
        Buffer = 9,
        Image = 10,
        Event = 11,
        QueryPool = 12,
        BufferView = 13,
        ImageView = 14,
        ShaderModule = 15,
        PipelineCache = 16,
        PipelineLayout = 17,
        RenderPass = 18,
        Pipeline = 19,
        DescriptorSetLayout = 20,
        Sampler = 21,
        DescriptorPool = 22,
        DescriptorSet = 23,
        Framebuffer = 24,
        CommandPool = 25,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum Format {
        Rgba8Srgb = 43,
        Bgra8Srgb = 50,
        R16Uint = 74,
        R32Uint = 98,
        R32Sfloat = 100,
        Rg32Sfloat = 103,
        Rgb32Uint = 104,
        Rgb32Sfloat = 106,
        Rgba32Sfloat = 109,
        D32Sfloat = 126,
    }

    impl_from_enum!(
        Format,
        Rgba8Srgb,
        Bgra8Srgb,
        R16Uint,
        R32Uint,
        R32Sfloat,
        Rg32Sfloat,
        Rgb32Uint,
        Rgb32Sfloat,
        Rgba32Sfloat,
        D32Sfloat
    );

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum IndexType {
        Uint16 = 0,
        Uint32 = 1,
    }

    impl From<super::IndexType> for IndexType {
        fn from(index_type: super::IndexType) -> Self {
            match index_type {
                super::IndexType::Uint16 => Self::Uint16,
                super::IndexType::Uint32 => Self::Uint32,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum ColorSpace {
        SrgbNonlinear = 0,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum PresentMode {
        Immediate = 0,
        Mailbox = 1,
        Fifo = 2,
        FifoRelaxed = 3,
    }

    pub type Extent2d = [u32; 2];
    pub type Extent3d = [u32; 3];

    pub type Offset2d = [i32; 2];
    pub type Offset3d = [i32; 3];

    pub type Bool = u32;

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SurfaceCapabilities {
        pub min_image_count: u32,
        pub max_image_count: u32,
        pub current_extent: Extent2d,
        pub min_image_extent: Extent2d,
        pub max_image_extent: Extent2d,
        pub max_image_array_layers: u32,
        pub supported_transforms: u32,
        pub current_transform: u32,
        pub supported_composite_alpha: u32,
        pub supported_usage_flags: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SurfaceFormat {
        pub format: Format,
        pub color_space: ColorSpace,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum SharingMode {
        Exclusive = 0,
        Concurrent = 1,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum CompositeAlpha {
        Opaque = 0x00000001,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ApplicationInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub application_name: *const i8,
        pub application_version: u32,
        pub engine_name: *const i8,
        pub engine_version: u32,
        pub api_version: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct InstanceCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub application_info: *const ApplicationInfo,
        pub enabled_layer_count: u32,
        pub enabled_layer_names: *const *const i8,
        pub enabled_extension_count: u32,
        pub enabled_extension_names: *const *const i8,
    }
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DebugUtilsLabel {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub label_name: *const i8,
        pub color: [f32; 4],
    }
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DebugUtilsObjectNameInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub object_type: ObjectType,
        pub object_handle: u64,
        pub object_name: *const i8,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DebugUtilsMessengerCallbackData {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub message_id_name: *const i8,
        pub message_id_number: i32,
        pub message: *const i8,
        pub queue_label_count: u32,
        pub queue_labels: *const DebugUtilsLabel,
        pub cmd_buf_label_count: u32,
        pub cmd_buf_labels: *const DebugUtilsLabel,
        pub object_count: u32,
        pub objects: *const DebugUtilsObjectNameInfo,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DebugUtilsMessengerCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub message_severity: i32,
        pub message_type: i32,
        pub user_callback: DebugUtilsMessengerCallbackInternal,
        pub user_data: *const (),
    }

    pub type DebugUtilsMessengerCallbackInternal = unsafe extern "system" fn(
        u32,
        u32,
        *const DebugUtilsMessengerCallbackData,
        *const (),
    ) -> Bool;

    pub type CreateDebugUtilsMessenger = unsafe extern "system" fn(
        Instance,
        *const DebugUtilsMessengerCreateInfo,
        *const (),
        *mut DebugUtilsMessenger,
    ) -> Result;

    pub type DestroyDebugUtilsMessenger =
        unsafe extern "system" fn(Instance, DebugUtilsMessenger, *const ()) -> Result;

    pub unsafe extern "system" fn debug_utils_messenger_callback(
        message_severity: u32,
        message_type: u32,
        callback_data: *const DebugUtilsMessengerCallbackData,
        user_data: *const (),
    ) -> Bool {
        let callback_data = callback_data.as_ref().unwrap();

        let f = mem::transmute::<_, super::DebugUtilsMessengerCallback>(user_data);

        let message = CStr::from_ptr(callback_data.message)
            .to_string_lossy()
            .into_owned();

        let exposed_callback_data = super::DebugUtilsMessengerCallbackData {
            message_severity,
            message_type,
            message: &message,
        };

        f(&exposed_callback_data) as _
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum PhysicalDeviceType {
        Other = 0,
        Integrated = 1,
        Discrete = 2,
        Virtual = 3,
        Cpu = 4,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PhysicalDeviceLimits {
        pub max_image_dimension_1d: u32,
        pub max_image_dimension_2d: u32,
        pub max_image_dimension_3d: u32,
        pub max_image_dimension_cube: u32,
        pub max_image_array_layers: u32,
        pub max_texel_buffer_elements: u32,
        pub max_uniform_buffer_range: u32,
        pub max_storage_buffer_range: u32,
        pub max_push_constants_size: u32,
        pub max_memory_allocation_count: u32,
        pub max_sampler_allocation_count: u32,
        pub buffer_image_granularity: DeviceSize,
        pub sparse_address_space_size: DeviceSize,
        pub max_bound_descriptor_sets: u32,
        pub max_per_stage_descriptor_samplers: u32,
        pub max_per_stage_descriptor_uniform_buffers: u32,
        pub max_per_stage_descriptor_storage_buffers: u32,
        pub max_per_stage_descriptor_sampled_images: u32,
        pub max_per_stage_descriptor_storage_images: u32,
        pub max_per_stage_descriptor_input_attachments: u32,
        pub max_per_stage_resources: u32,
        pub max_descriptor_set_samplers: u32,
        pub max_descriptor_set_uniform_buffers: u32,
        pub max_descriptor_set_uniform_buffers_dynamic: u32,
        pub max_descriptor_set_storage_buffers: u32,
        pub max_descriptor_set_storage_buffers_dynamic: u32,
        pub max_descriptor_set_sampled_images: u32,
        pub max_descriptor_set_storage_images: u32,
        pub max_descriptor_set_input_attachments: u32,
        pub max_vertex_input_attributes: u32,
        pub max_vertex_input_bindings: u32,
        pub max_vertex_input_attribute_offset: u32,
        pub max_vertex_input_binding_stride: u32,
        pub max_vertex_output_components: u32,
        pub max_tessellation_generation_level: u32,
        pub max_tessellation_patch_size: u32,
        pub max_tessellation_control_per_vertex_input_components: u32,
        pub max_tessellation_control_per_vertex_output_components: u32,
        pub max_tessellation_control_per_patch_output_components: u32,
        pub max_tessellation_control_total_output_components: u32,
        pub max_tessellation_evaluation_input_components: u32,
        pub max_tessellation_evaluation_output_components: u32,
        pub max_geometry_shader_invocations: u32,
        pub max_geometry_input_components: u32,
        pub max_geometry_output_components: u32,
        pub max_geometry_output_vertices: u32,
        pub max_geometry_total_output_components: u32,
        pub max_fragment_input_components: u32,
        pub max_fragment_output_attachments: u32,
        pub max_fragment_dual_src_attachments: u32,
        pub max_fragment_combined_output_resources: u32,
        pub max_compute_shared_memory_size: u32,
        pub max_compute_work_group_count: [u32; 3],
        pub max_compute_work_group_invocations: u32,
        pub max_compute_work_group_size: [u32; 3],
        pub sub_pixel_precision_bits: u32,
        pub sub_texel_precision_bits: u32,
        pub mipmap_precision_bits: u32,
        pub max_draw_indexed_index_value: u32,
        pub max_draw_indirect_count: u32,
        pub max_sampler_lod_bias: f32,
        pub max_sampler_anisotropy: f32,
        pub max_viewports: u32,
        pub max_viewport_dimensions: [u32; 2],
        pub viewport_bounds_range: [f32; 2],
        pub viewport_sub_pixel_bits: u32,
        pub min_memory_map_alignment: usize,
        pub min_texel_buffer_offset_alignment: DeviceSize,
        pub min_uniform_buffer_offset_alignment: DeviceSize,
        pub min_storage_buffer_offset_alignment: DeviceSize,
        pub min_texel_offset: i32,
        pub max_texel_offset: u32,
        pub min_texel_gather_offset: i32,
        pub max_texel_gather_offset: u32,
        pub min_interpolation_offset: f32,
        pub max_interpolation_offset: f32,
        pub sub_pixel_interpolation_offset_bits: u32,
        pub max_framebuffer_width: u32,
        pub max_framebuffer_height: u32,
        pub max_framebuffer_layers: u32,
        pub framebuffer_color_sample_counts: Flags,
        pub framebuffer_depth_sample_counts: Flags,
        pub framebuffer_stencil_sample_counts: Flags,
        pub framebuffer_no_attachments_sample_counts: Flags,
        pub max_color_attachments: u32,
        pub sampled_image_color_sample_counts: Flags,
        pub sampled_image_integer_sample_counts: Flags,
        pub sampled_imae_depth_sample_counts: Flags,
        pub sampled_image_stencil_sample_counts: Flags,
        pub storage_image_sample_counts: Flags,
        pub max_sample_mask_words: u32,
        pub timestamp_compute_and_graphics: Bool,
        pub timestamp_period: f32,
        pub max_clip_distances: u32,
        pub max_cull_distances: u32,
        pub max_combined_clip_and_cull_distances: u32,
        pub discrete_queue_priorities: u32,
        pub point_size_range: [f32; 2],
        pub line_width_range: [f32; 2],
        pub point_size_granularity: f32,
        pub line_width_granularity: f32,
        pub strict_lines: Bool,
        pub standard_sample_locations: Bool,
        pub optimal_buffer_copy_offset_alignment: DeviceSize,
        pub optimal_buffer_copy_row_pitch_alignment: DeviceSize,
        pub non_coherent_atom_size: DeviceSize,
    }
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PhysicalDeviceSparseProperties {
        pub residency_standard_2d_block_shape: Bool,
        pub residency_standard_2d_multisample_block_shape: Bool,
        pub residency_standard_3d_block_shape: Bool,
        pub residency_aligned_mip_size: Bool,
        pub residency_non_resident_strict: Bool,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PhysicalDeviceProperties {
        pub api_version: u32,
        pub driver_version: u32,
        pub vendor_id: u32,
        pub device_id: u32,
        pub device_type: PhysicalDeviceType,
        pub device_name: [i8; 256],
        pub pipeline_cache_uuid: [i8; 16],
        pub limits: PhysicalDeviceLimits,
        pub sparse_properties: PhysicalDeviceSparseProperties,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PhysicalDeviceFeatures {
        pub robust_buffer_access: Bool,
        pub full_draw_index_uint_32: Bool,
        pub image_cube_array: Bool,
        pub independent_blend: Bool,
        pub geometry_shader: Bool,
        pub tessellation_shader: Bool,
        pub sample_rate_shading: Bool,
        pub dual_src_blend: Bool,
        pub logic_op: Bool,
        pub multi_draw_indirect: Bool,
        pub draw_indirect_first_instance: Bool,
        pub depth_clamp: Bool,
        pub depth_bias_clamp: Bool,
        pub fill_mode_non_solid: Bool,
        pub depth_bounds: Bool,
        pub wide_lines: Bool,
        pub large_points: Bool,
        pub alpha_to_one: Bool,
        pub multi_viewport: Bool,
        pub sampler_anisotropy: Bool,
        pub texture_compression_etc_2: Bool,
        pub texture_compression_astc_ldr: Bool,
        pub texture_compression_bc: Bool,
        pub occlusion_query_precise: Bool,
        pub pipeline_statistics_query: Bool,
        pub vertex_pipeline_stores_and_atomics: Bool,
        pub fragment_stores_and_atomics: Bool,
        pub shader_tessellation_and_geometry_point_size: Bool,
        pub shader_image_gather_extended: Bool,
        pub shader_storage_image_extended_formats: Bool,
        pub shader_storage_image_multisample: Bool,
        pub shader_storage_image_read_without_format: Bool,
        pub shader_storage_image_write_without_format: Bool,
        pub shader_uniform_buffer_array_dynamic_indexing: Bool,
        pub shader_sampled_image_array_dynamic_indexing: Bool,
        pub shader_storage_buffer_array_dynamic_indexing: Bool,
        pub shader_storage_image_array_dynamic_indexing: Bool,
        pub shader_clip_distance: Bool,
        pub shader_cull_distance: Bool,
        pub shader_float_64: Bool,
        pub shader_int_64: Bool,
        pub shader_int_16: Bool,
        pub shader_resource_residency: Bool,
        pub shader_resource_min_lod: Bool,
        pub sparse_binding: Bool,
        pub sparse_residency_buffer: Bool,
        pub sparse_residency_image_2_d: Bool,
        pub sparse_residency_image_3_d: Bool,
        pub sparse_residency_2_samples: Bool,
        pub sparse_residency_4_samples: Bool,
        pub sparse_residency_8_samples: Bool,
        pub sparse_residency_16_samples: Bool,
        pub sparse_residency_aliased: Bool,
        pub variable_multisample_rate: Bool,
        pub inherited_queries: Bool,
    }

    impl_from_struct!(
        PhysicalDeviceFeatures,
        robust_buffer_access,
        full_draw_index_uint_32,
        image_cube_array,
        independent_blend,
        geometry_shader,
        tessellation_shader,
        sample_rate_shading,
        dual_src_blend,
        logic_op,
        multi_draw_indirect,
        draw_indirect_first_instance,
        depth_clamp,
        depth_bias_clamp,
        fill_mode_non_solid,
        depth_bounds,
        wide_lines,
        large_points,
        alpha_to_one,
        multi_viewport,
        sampler_anisotropy,
        texture_compression_etc_2,
        texture_compression_astc_ldr,
        texture_compression_bc,
        occlusion_query_precise,
        pipeline_statistics_query,
        vertex_pipeline_stores_and_atomics,
        fragment_stores_and_atomics,
        shader_tessellation_and_geometry_point_size,
        shader_image_gather_extended,
        shader_storage_image_extended_formats,
        shader_storage_image_multisample,
        shader_storage_image_read_without_format,
        shader_storage_image_write_without_format,
        shader_uniform_buffer_array_dynamic_indexing,
        shader_sampled_image_array_dynamic_indexing,
        shader_storage_buffer_array_dynamic_indexing,
        shader_storage_image_array_dynamic_indexing,
        shader_clip_distance,
        shader_cull_distance,
        shader_float_64,
        shader_int_64,
        shader_int_16,
        shader_resource_residency,
        shader_resource_min_lod,
        sparse_binding,
        sparse_residency_buffer,
        sparse_residency_image_2_d,
        sparse_residency_image_3_d,
        sparse_residency_2_samples,
        sparse_residency_4_samples,
        sparse_residency_8_samples,
        sparse_residency_16_samples,
        sparse_residency_aliased,
        variable_multisample_rate,
        inherited_queries
    );

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct QueueFamilyProperties {
        pub queue_flags: u32,
        pub queue_count: u32,
        pub timestamp_valid_bits: u32,
        pub min_image_transfer_granularity: [u32; 3],
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DeviceQueueCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub queue_family_index: u32,
        pub queue_count: u32,
        pub queue_priorities: *const f32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DeviceCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub queue_create_info_count: u32,
        pub queue_create_infos: *const DeviceQueueCreateInfo,
        pub enabled_layer_count: u32,
        pub enabled_layer_names: *const *const i8,
        pub enabled_extension_count: u32,
        pub enabled_extension_names: *const *const i8,
        pub enabled_features: *const PhysicalDeviceFeatures,
    }

    #[cfg(target_os = "linux")]
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct XlibSurfaceCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub display: *const (),
        pub window: u64,
    }

    #[cfg(target_os = "windows")]
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct Win32SurfaceCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub hinstance: *mut (),
        pub hwnd: *mut (),
    }
    
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SwapchainCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub surface: Surface,
        pub min_image_count: u32,
        pub image_format: Format,
        pub image_color_space: ColorSpace,
        pub image_extent: Extent2d,
        pub image_array_layers: u32,
        pub image_usage: u32,
        pub image_sharing_mode: SharingMode,
        pub queue_family_index_count: u32,
        pub queue_family_indices: *const u32,
        pub pre_transform: u32,
        pub composite_alpha: CompositeAlpha,
        pub present_mode: PresentMode,
        pub clipped: Bool,
        pub old_swapchain: Swapchain,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum ImageViewType {
        OneDim = 0,
        TwoDim = 1,
        ThreeDim = 2,
        Cube = 3,
        OneDimArray = 4,
        TwoDimArray = 5,
        ThreeDimArray = 6,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum ComponentSwizzle {
        Identity = 0,
        Zero = 1,
        One = 2,
        R = 3,
        G = 4,
        B = 5,
        A = 6,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ComponentMapping {
        pub r: ComponentSwizzle,
        pub g: ComponentSwizzle,
        pub b: ComponentSwizzle,
        pub a: ComponentSwizzle,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ImageSubresourceRange {
        pub aspect_mask: u32,
        pub base_mip_level: u32,
        pub level_count: u32,
        pub base_array_layer: u32,
        pub layer_count: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ImageViewCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub image: Image,
        pub view_type: ImageViewType,
        pub format: Format,
        pub components: ComponentMapping,
        pub subresource_range: ImageSubresourceRange,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ShaderModuleCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub code_size: usize,
        pub code: *const u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineShaderStageCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub stage: u32,
        pub module: ShaderModule,
        pub entry_point: *const i8,
        pub specialization_info: *const (),
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PushConstantRange {
        pub stage_flags: u32,
        pub offset: u32,
        pub size: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineLayoutCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub set_layout_count: u32,
        pub set_layouts: *const DescriptorSetLayout,
        pub push_constant_range_count: u32,
        pub push_constant_ranges: *const PushConstantRange,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum AttachmentLoadOp {
        Load = 0,
        Clear = 1,
        DontCare = 2,
    }

    impl From<super::AttachmentLoadOp> for AttachmentLoadOp {
        fn from(load_op: super::AttachmentLoadOp) -> Self {
            match load_op {
                super::AttachmentLoadOp::Load => Self::Load,
                super::AttachmentLoadOp::Clear => Self::Clear,
                super::AttachmentLoadOp::DontCare => Self::DontCare,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum AttachmentStoreOp {
        Store = 0,
        DontCare = 1,
    }

    impl From<super::AttachmentStoreOp> for AttachmentStoreOp {
        fn from(store_op: super::AttachmentStoreOp) -> Self {
            match store_op {
                super::AttachmentStoreOp::Store => Self::Store,
                super::AttachmentStoreOp::DontCare => Self::DontCare,
            }
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    #[repr(C)]
    pub enum ImageLayout {
        Undefined = 0,
        General = 1,
        ColorAttachment = 2,
        DepthStencilAttachment = 3,
        DepthStencilReadOnly = 4,
        ShaderReadOnly = 5,
        TransferSrc = 6,
        TransferDst = 7,
        Preinitialized = 8,
        PresentSrc = 1000001002,
    }

    impl From<super::ImageLayout> for ImageLayout {
        fn from(image_layout: super::ImageLayout) -> Self {
            match image_layout {
                super::ImageLayout::Undefined => Self::Undefined,
                super::ImageLayout::General => Self::General,
                super::ImageLayout::ColorAttachment => Self::ColorAttachment,
                super::ImageLayout::DepthStencilAttachment => Self::DepthStencilAttachment,
                super::ImageLayout::DepthStencilReadOnly => Self::DepthStencilReadOnly,
                super::ImageLayout::ShaderReadOnly => Self::ShaderReadOnly,
                super::ImageLayout::TransferSrc => Self::TransferSrc,
                super::ImageLayout::TransferDst => Self::TransferDst,
                super::ImageLayout::Preinitialized => Self::Preinitialized,
                super::ImageLayout::PresentSrc => Self::PresentSrc,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct AttachmentDescription {
        flags: u32,
        format: Format,
        samples: u32,
        load_op: AttachmentLoadOp,
        store_op: AttachmentStoreOp,
        stencil_load_op: AttachmentLoadOp,
        stencil_store_op: AttachmentStoreOp,
        initial_layout: ImageLayout,
        final_layout: ImageLayout,
    }

    impl From<super::AttachmentDescription> for AttachmentDescription {
        fn from(attachment_description: super::AttachmentDescription) -> Self {
            Self {
                flags: 0,
                format: attachment_description.format.into(),
                samples: attachment_description.samples as _,
                load_op: attachment_description.load_op.into(),
                store_op: attachment_description.store_op.into(),
                stencil_load_op: attachment_description.stencil_load_op.into(),
                stencil_store_op: attachment_description.stencil_store_op.into(),
                initial_layout: attachment_description.initial_layout.into(),
                final_layout: attachment_description.final_layout.into(),
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct AttachmentReference {
        pub attachment: u32,
        pub layout: ImageLayout,
    }

    impl From<super::AttachmentReference> for AttachmentReference {
        fn from(attachment_reference: super::AttachmentReference) -> Self {
            Self {
                attachment: attachment_reference.attachment as _,
                layout: attachment_reference.layout.into(),
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum PipelineBindPoint {
        Graphics = 0,
        Compute = 1,
    }

    impl From<super::PipelineBindPoint> for PipelineBindPoint {
        fn from(pipeline_bind_point: super::PipelineBindPoint) -> Self {
            match pipeline_bind_point {
                super::PipelineBindPoint::Graphics => Self::Graphics,
                super::PipelineBindPoint::Compute => Self::Compute,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SubpassDescription {
        pub flags: u32,
        pub pipeline_bind_point: PipelineBindPoint,
        pub input_attachment_count: u32,
        pub input_attachments: *const AttachmentReference,
        pub color_attachment_count: u32,
        pub color_attachments: *const AttachmentReference,
        pub resolve_attachments: *const AttachmentReference,
        pub depth_stencil_attachment: *const AttachmentReference,
        pub preserve_attachment_count: u32,
        pub preserve_attachments: *const u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SubpassDependency {
        pub src_subpass: u32,
        pub dst_subpass: u32,
        pub src_stage_mask: u32,
        pub dst_stage_mask: u32,
        pub src_access_mask: u32,
        pub dst_access_mask: u32,
        pub dependency_flags: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct RenderPassCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub attachment_count: u32,
        pub attachments: *const AttachmentDescription,
        pub subpass_count: u32,
        pub subpasses: *const SubpassDescription,
        pub dependency_count: u32,
        pub dependencies: *const SubpassDependency,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum VertexInputRate {
        Vertex = 0,
        Instance = 1,
    }

    impl From<super::VertexInputRate> for VertexInputRate {
        fn from(input_rate: super::VertexInputRate) -> Self {
            match input_rate {
                super::VertexInputRate::Vertex => Self::Vertex,
                super::VertexInputRate::Instance => Self::Instance,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct VertexInputBindingDescription {
        pub binding: u32,
        pub stride: u32,
        pub input_rate: VertexInputRate,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct VertexInputAttributeDescription {
        pub location: u32,
        pub binding: u32,
        pub format: Format,
        pub offset: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineVertexInputStateCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub vertex_binding_description_count: u32,
        pub vertex_binding_descriptions: *const VertexInputBindingDescription,
        pub vertex_attribute_description_count: u32,
        pub vertex_attribute_descriptions: *const VertexInputAttributeDescription,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum PrimitiveTopology {
        PointList = 0,
        LineList = 1,
        LineStrip = 2,
        TriangleList = 3,
        TriangleStrip = 4,
    }

    impl From<super::PrimitiveTopology> for PrimitiveTopology {
        fn from(topology: super::PrimitiveTopology) -> Self {
            match topology {
                super::PrimitiveTopology::PointList => Self::PointList,
                super::PrimitiveTopology::LineList => Self::LineList,
                super::PrimitiveTopology::LineStrip => Self::LineStrip,
                super::PrimitiveTopology::TriangleList => Self::TriangleList,
                super::PrimitiveTopology::TriangleStrip => Self::TriangleStrip,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineInputAssemblyStateCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub topology: PrimitiveTopology,
        pub primitive_restart_enable: Bool,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineTessellationStateCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub patch_control_points: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct Viewport {
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
        pub min_depth: f32,
        pub max_depth: f32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct Rect2d {
        pub offset: Offset2d,
        pub extent: Extent2d,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineViewportStateCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub viewport_count: u32,
        pub viewports: *const Viewport,
        pub scissor_count: u32,
        pub scissors: *const Rect2d,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum PolygonMode {
        Fill = 0,
        Line = 1,
        Point = 2,
    }

    impl From<super::PolygonMode> for PolygonMode {
        fn from(polygon_mode: super::PolygonMode) -> Self {
            match polygon_mode {
                super::PolygonMode::Fill => Self::Fill,
                super::PolygonMode::Line => Self::Line,
                super::PolygonMode::Point => Self::Point,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum FrontFace {
        CounterClockwise = 0,
        Clockwise = 1,
    }

    impl From<super::FrontFace> for FrontFace {
        fn from(front_face: super::FrontFace) -> Self {
            match front_face {
                super::FrontFace::CounterClockwise => Self::CounterClockwise,
                super::FrontFace::Clockwise => Self::Clockwise,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineRasterizationStateCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub depth_clamp_enable: Bool,
        pub rasterizer_discard_enable: Bool,
        pub polygon_mode: PolygonMode,
        pub cull_mode: u32,
        pub front_face: FrontFace,
        pub depth_bias_enable: Bool,
        pub depth_bias_constant_factor: f32,
        pub depth_bias_clamp: f32,
        pub depth_bias_slope_factor: f32,
        pub line_width: f32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineMultisampleStateCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub rasterization_samples: u32,
        pub sample_shading_enable: Bool,
        pub min_sample_shading: f32,
        pub sample_mask: *const u32,
        pub alpha_to_coverage_enable: Bool,
        pub alpha_to_one_enable: Bool,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum CompareOp {
        Never = 0,
        Less = 1,
        Equal = 2,
        LessOrEqual = 3,
        Greater = 4,
        NotEqual = 5,
        GreaterOrEqual = 6,
        Always = 7,
    }

    impl_from_enum!(
        CompareOp,
        Never,
        Less,
        Equal,
        LessOrEqual,
        Greater,
        NotEqual,
        GreaterOrEqual,
        Always
    );

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum StencilOp {
        Keep = 0,
        Zero = 1,
        Replace = 2,
        IncrementAndClamp = 3,
        DecrementAndClamp = 4,
        Invert = 5,
        IncrementAndWrap = 6,
        DecrementAndWrap = 7,
    }

    impl_from_enum!(
        StencilOp,
        Keep,
        Zero,
        Replace,
        IncrementAndClamp,
        DecrementAndClamp,
        Invert,
        IncrementAndWrap,
        DecrementAndWrap
    );

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct StencilOpState {
        pub fail_op: StencilOp,
        pub pass_op: StencilOp,
        pub depth_fail_op: StencilOp,
        pub compare_op: CompareOp,
        pub compare_mask: u32,
        pub write_mask: u32,
        pub reference: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineDepthStencilStateCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub depth_test_enable: Bool,
        pub depth_write_enable: Bool,
        pub depth_compare_op: CompareOp,
        pub depth_bounds_test_enable: Bool,
        pub stencil_test_enable: Bool,
        pub front: StencilOpState,
        pub back: StencilOpState,
        pub min_depth_bounds: f32,
        pub max_depth_bounds: f32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum BlendFactor {
        Zero = 0,
        One = 1,
        SrcAlpha = 6,
        OneMinusSrcAlpha = 7,
    }

    impl From<super::BlendFactor> for BlendFactor {
        fn from(blend_factor: super::BlendFactor) -> Self {
            match blend_factor {
                super::BlendFactor::Zero => Self::Zero,
                super::BlendFactor::One => Self::One,
                super::BlendFactor::SrcAlpha => Self::SrcAlpha,
                super::BlendFactor::OneMinusSrcAlpha => Self::OneMinusSrcAlpha,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum BlendOp {
        Add = 0,
    }

    impl From<super::BlendOp> for BlendOp {
        fn from(blend_op: super::BlendOp) -> Self {
            match blend_op {
                super::BlendOp::Add => Self::Add,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineColorBlendAttachmentState {
        pub blend_enable: Bool,
        pub src_color_blend_factor: BlendFactor,
        pub dst_color_blend_factor: BlendFactor,
        pub color_blend_op: BlendOp,
        pub src_alpha_blend_factor: BlendFactor,
        pub dst_alpha_blend_factor: BlendFactor,
        pub alpha_blend_op: BlendOp,
        pub color_write_mask: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum LogicOp {
        Copy = 3,
    }

    impl From<super::LogicOp> for LogicOp {
        fn from(logic_op: super::LogicOp) -> Self {
            match logic_op {
                super::LogicOp::Copy => Self::Copy,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineColorBlendStateCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub logic_op_enable: Bool,
        pub logic_op: LogicOp,
        pub attachment_count: u32,
        pub attachments: *const PipelineColorBlendAttachmentState,
        pub blend_constants: [f32; 4],
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum DynamicState {
        Viewport = 0,
    }

    impl From<super::DynamicState> for DynamicState {
        fn from(dynamic_state: super::DynamicState) -> Self {
            match dynamic_state {
                super::DynamicState::Viewport => Self::Viewport,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineDynamicStateCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub dynamic_state_count: u32,
        pub dynamic_states: *const DynamicState,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ComputePipelineCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub stage: PipelineShaderStageCreateInfo,
        pub layout: PipelineLayout,
        pub base_pipeline_handle: Pipeline,
        pub base_pipeline_index: i32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct GraphicsPipelineCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub stage_count: u32,
        pub stages: *const PipelineShaderStageCreateInfo,
        pub vertex_input_state: *const PipelineVertexInputStateCreateInfo,
        pub input_assembly_state: *const PipelineInputAssemblyStateCreateInfo,
        pub tessellation_state: *const PipelineTessellationStateCreateInfo,
        pub viewport_state: *const PipelineViewportStateCreateInfo,
        pub rasterization_state: *const PipelineRasterizationStateCreateInfo,
        pub multisample_state: *const PipelineMultisampleStateCreateInfo,
        pub depth_stencil_state: *const PipelineDepthStencilStateCreateInfo,
        pub color_blend_state: *const PipelineColorBlendStateCreateInfo,
        pub dynamic_state: *const PipelineDynamicStateCreateInfo,
        pub layout: PipelineLayout,
        pub render_pass: RenderPass,
        pub subpass: u32,
        pub base_pipeline_handle: Pipeline,
        pub base_pipeline_index: i32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct FramebufferCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub render_pass: RenderPass,
        pub attachment_count: u32,
        pub attachments: *const ImageView,
        pub width: u32,
        pub height: u32,
        pub layers: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ClearColorValue {
        pub rgba: [f32; 4],
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ClearDepthStencilValue {
        pub depth: f32,
        pub stencil: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub union ClearValue {
        pub color: ClearColorValue,
        pub depth_stencil: ClearDepthStencilValue,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct RenderPassBeginInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub render_pass: RenderPass,
        pub framebuffer: Framebuffer,
        pub render_area: Rect2d,
        pub clear_value_count: u32,
        pub clear_values: *const ClearValue,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum SubpassContents {
        Inline = 0,
        Secondary = 1,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct CommandBufferBeginInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub inheritence_info: *const (),
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct CommandPoolCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
        pub queue_family_index: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum CommandBufferLevel {
        Primary = 0,
        Secondary = 1,
    }

    impl_from_enum!(CommandBufferLevel, Primary, Secondary);

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct CommandBufferAllocateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub command_pool: CommandPool,
        pub level: CommandBufferLevel,
        pub command_buffer_count: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct FenceCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SemaphoreCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SubmitInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub wait_semaphore_count: u32,
        pub wait_semaphores: *const Semaphore,
        pub wait_dst_stage_mask: *const Flags,
        pub command_buffer_count: u32,
        pub command_buffers: *const CommandBuffer,
        pub signal_semaphore_count: u32,
        pub signal_semaphores: *const Semaphore,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PresentInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub wait_semaphore_count: u32,
        pub wait_semaphores: *const Semaphore,
        pub swapchain_count: u32,
        pub swapchains: *const Swapchain,
        pub image_indices: *const u32,
        pub results: *const Result,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct BufferCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: Flags,
        pub size: DeviceSize,
        pub usage: Flags,
        pub sharing_mode: SharingMode,
        pub queue_family_index_count: u32,
        pub queue_family_indices: *const u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct MemoryAllocateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub size: DeviceSize,
        pub memory_type_index: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct MemoryRequirements {
        pub size: DeviceSize,
        pub alignment: DeviceSize,
        pub memory_type: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct MemoryType {
        pub property_flags: Flags,
        pub heap_index: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct MemoryHeap {
        pub size: DeviceSize,
        pub flags: Flags,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PhysicalDeviceMemoryProperties {
        pub memory_type_count: u32,
        pub memory_types: [MemoryType; 32],
        pub memory_heap_count: u32,
        pub memory_heaps: [MemoryHeap; 32],
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum DescriptorType {
        CombinedImageSampler = 1,
        StorageImage = 3,
        UniformBuffer = 6,
        StorageBuffer = 7,
    }

    impl_from_enum!(
        DescriptorType,
        CombinedImageSampler,
        StorageImage,
        UniformBuffer,
        StorageBuffer
    );

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DescriptorSetLayoutBinding {
        pub binding: u32,
        pub descriptor_type: DescriptorType,
        pub descriptor_count: u32,
        pub stage: u32,
        pub immutable_samplers: *const (),
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DescriptorSetLayoutCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: Flags,
        pub binding_count: u32,
        pub bindings: *const DescriptorSetLayoutBinding,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DescriptorBufferInfo {
        pub buffer: Buffer,
        pub offset: DeviceSize,
        pub range: DeviceSize,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DescriptorImageInfo {
        pub sampler: Sampler,
        pub image_view: ImageView,
        pub image_layout: ImageLayout,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct WriteDescriptorSet {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub dst_set: DescriptorSet,
        pub dst_binding: u32,
        pub dst_array_element: u32,
        pub descriptor_count: u32,
        pub descriptor_type: DescriptorType,
        pub image_infos: *const DescriptorImageInfo,
        pub buffer_infos: *const DescriptorBufferInfo,
        pub texel_buffer_view: *const (),
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct CopyDescriptorSet {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub src_set: DescriptorSet,
        pub src_binding: u32,
        pub src_array_element: u32,
        pub dst_set: DescriptorSet,
        pub dst_binding: u32,
        pub dst_array_element: u32,
        pub descriptor_count: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DescriptorPoolSize {
        pub descriptor_type: DescriptorType,
        pub descriptor_count: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DescriptorSetAllocateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub descriptor_pool: DescriptorPool,
        pub descriptor_set_count: u32,
        pub set_layouts: *const DescriptorSetLayout,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DescriptorPoolCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: Flags,
        pub max_sets: u32,
        pub pool_size_count: u32,
        pub pool_sizes: *const DescriptorPoolSize,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum ImageType {
        OneDim = 0,
        TwoDim = 1,
        ThreeDim = 2,
    }

    impl From<super::ImageType> for ImageType {
        fn from(image_type: super::ImageType) -> Self {
            match image_type {
                super::ImageType::OneDim => Self::OneDim,
                super::ImageType::TwoDim => Self::TwoDim,
                super::ImageType::ThreeDim => Self::ThreeDim,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum ImageTiling {
        Optimal = 0,
        Linear = 1,
    }

    impl From<super::ImageTiling> for ImageTiling {
        fn from(image_tiling: super::ImageTiling) -> Self {
            match image_tiling {
                super::ImageTiling::Optimal => Self::Optimal,
                super::ImageTiling::Linear => Self::Linear,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ImageCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: Flags,
        pub image_type: ImageType,
        pub format: Format,
        pub extent: Extent3d,
        pub mip_levels: u32,
        pub array_layers: u32,
        pub samples: Flags,
        pub tiling: ImageTiling,
        pub image_usage: u32,
        pub sharing_mode: SharingMode,
        pub queue_family_index_count: u32,
        pub queue_family_indices: *const u32,
        pub initial_layout: ImageLayout,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct BufferCopy {
        pub src_offset: DeviceSize,
        pub dst_offset: DeviceSize,
        pub size: DeviceSize,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ImageSubresourceLayers {
        pub aspect_mask: Flags,
        pub mip_level: u32,
        pub base_array_layer: u32,
        pub layer_count: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct BufferImageCopy {
        pub buffer_offset: DeviceSize,
        pub buffer_row_length: u32,
        pub buffer_image_height: u32,
        pub image_subresource: ImageSubresourceLayers,
        pub image_offset: Offset3d,
        pub image_extent: Extent3d,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum Filter {
        Nearest = 0,
        Linear = 1,
    }

    impl From<super::Filter> for Filter {
        fn from(filter: super::Filter) -> Self {
            match filter {
                super::Filter::Nearest => Self::Nearest,
                super::Filter::Linear => Self::Linear,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum SamplerMipmapMode {
        Nearest = 0,
        Linear = 1,
    }

    impl From<super::SamplerMipmapMode> for SamplerMipmapMode {
        fn from(mipmap_mode: super::SamplerMipmapMode) -> Self {
            match mipmap_mode {
                super::SamplerMipmapMode::Nearest => Self::Nearest,
                super::SamplerMipmapMode::Linear => Self::Linear,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum SamplerAddressMode {
        Repeat = 0,
        MirroredRepeat = 1,
        ClampToEdge = 2,
        ClampToBorder = 3,
    }

    impl From<super::SamplerAddressMode> for SamplerAddressMode {
        fn from(address_mode: super::SamplerAddressMode) -> Self {
            match address_mode {
                super::SamplerAddressMode::Repeat => Self::Repeat,
                super::SamplerAddressMode::MirroredRepeat => Self::MirroredRepeat,
                super::SamplerAddressMode::ClampToEdge => Self::ClampToEdge,
                super::SamplerAddressMode::ClampToBorder => Self::ClampToBorder,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum BorderColor {
        FloatTransparentBlack = 0,
        IntTransparentBlack = 1,
        FloatOpaqueBlack = 2,
        IntOpaqueBlack = 3,
        FloatOpaqueWhite = 4,
        IntOpaqueWhite = 5,
    }

    impl From<super::BorderColor> for BorderColor {
        fn from(border_color: super::BorderColor) -> Self {
            match border_color {
                super::BorderColor::FloatTransparentBlack => Self::FloatTransparentBlack,
                super::BorderColor::IntTransparentBlack => Self::IntTransparentBlack,
                super::BorderColor::FloatOpaqueBlack => Self::FloatOpaqueBlack,
                super::BorderColor::IntOpaqueBlack => Self::IntOpaqueBlack,
                super::BorderColor::FloatOpaqueWhite => Self::FloatOpaqueWhite,
                super::BorderColor::IntOpaqueWhite => Self::IntOpaqueWhite,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SamplerCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub flags: Flags,
        pub mag_filter: Filter,
        pub min_filter: Filter,
        pub mipmap_mode: SamplerMipmapMode,
        pub address_mode_u: SamplerAddressMode,
        pub address_mode_v: SamplerAddressMode,
        pub address_mode_w: SamplerAddressMode,
        pub mip_lod_bias: f32,
        pub anisotropy_enable: Bool,
        pub max_anisotropy: f32,
        pub compare_enable: Bool,
        pub compare_op: CompareOp,
        pub min_lod: f32,
        pub max_lod: f32,
        pub border_color: BorderColor,
        pub unnormalized_coordinates: Bool,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ImageMemoryBarrier {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub src_access_mask: u32,
        pub dst_access_mask: u32,
        pub old_layout: ImageLayout,
        pub new_layout: ImageLayout,
        pub src_queue_family_index: u32,
        pub dst_queue_family_index: u32,
        pub image: Image,
        pub subresource_range: ImageSubresourceRange,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct MemoryBarrier {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub src_access_mask: u32,
        pub dst_access_mask: u32,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct BufferMemoryBarrier {
        pub structure_type: StructureType,
        pub p_next: *const (),
        pub src_access_mask: u32,
        pub dst_access_mask: u32,
        pub src_queue_family_index: u32,
        pub dst_queue_family_index: u32,
        pub buffer: Buffer,
        pub offset: u64,
        pub size: u64,
    }

    #[link(name = "vulkan-1")]
    #[allow(non_snake_case)]
    extern "C" {
        //TODO implement VkAllocationCallbacks
        pub fn vkCreateInstance(
            create_info: *const InstanceCreateInfo,
            allocator: *const (),
            instance: *mut Instance,
        ) -> Result;
        pub fn vkDestroyInstance(instance: Instance, allocator: *const ());
        pub fn vkGetInstanceProcAddr(instance: Instance, name: *const i8) -> *const ();
        pub fn vkEnumeratePhysicalDevices(
            instance: Instance,
            physical_device_count: *mut u32,
            physical_devices: *mut PhysicalDevice,
        ) -> Result;
        pub fn vkGetPhysicalDeviceProperties(
            physical_device: PhysicalDevice,
            properties: *mut PhysicalDeviceProperties,
        );
        pub fn vkGetPhysicalDeviceQueueFamilyProperties(
            physical_device: PhysicalDevice,
            queue_family_property_count: *mut u32,
            queue_family_properties: *mut QueueFamilyProperties,
        );
        pub fn vkGetPhysicalDeviceSurfaceCapabilitiesKHR(
            physical_device: PhysicalDevice,
            surface: Surface,
            surface_capabilities: *mut SurfaceCapabilities,
        );
        pub fn vkGetPhysicalDeviceSurfaceSupportKHR(
            physical_device: PhysicalDevice,
            queue_family_index: u32,
            surface: Surface,
            supported: *mut Bool,
        ) -> Result;
        pub fn vkGetPhysicalDeviceMemoryProperties(
            physical_device: PhysicalDevice,
            memory_properties: *mut PhysicalDeviceMemoryProperties,
        );
        pub fn vkCreateDevice(
            physical_device: PhysicalDevice,
            create_info: *const DeviceCreateInfo,
            allocator: *const (),
            device: *mut Device,
        ) -> Result;
        pub fn vkDestroyDevice(device: Device, allocator: *const ());
        pub fn vkDeviceWaitIdle(device: Device) -> Result;
        pub fn vkQueueWaitIdle(queue: Queue) -> Result;
        pub fn vkGetDeviceQueue(
            device: Device,
            queue_family_index: u32,
            queue_index: u32,
            queue: *mut Queue,
        );
        pub fn vkGetBufferMemoryRequirements(
            device: Device,
            buffer: Buffer,
            memory_requirements: *mut MemoryRequirements,
        );
        #[cfg(target_os = "linux")]
        pub fn vkCreateXlibSurfaceKHR(
            instance: Instance,
            create_info: *const XlibSurfaceCreateInfo,
            allocator: *const (),
            surface: *mut Surface,
        );
        #[cfg(target_os = "windows")]
        pub fn vkCreateWin32SurfaceKHR(
            instance: Instance,
            create_info: *const Win32SurfaceCreateInfo,
            allocator: *const (),
            surface: *mut Surface,
        );
        pub fn vkDestroySurfaceKHR(instance: Instance, surface: Surface, allocator: *const ());
        pub fn vkCreateSwapchainKHR(
            device: Device,
            create_info: *const SwapchainCreateInfo,
            allocator: *const (),
            swapchain: *mut Swapchain,
        ) -> Result;
        pub fn vkDestroySwapchainKHR(
            device: Device,
            swapchain: Swapchain,
            allocator: *const (),
        );
        pub fn vkGetSwapchainImagesKHR(
            device: Device,
            swapchain: Swapchain,
            swapchain_image_count: *mut u32,
            swapchain_images: *mut Image,
        );
        pub fn vkCreateImageView(
            device: Device,
            create_info: *const ImageViewCreateInfo,
            allocator: *const (),
            image_view: *mut ImageView,
        ) -> Result;
        pub fn vkDestroyImageView(device: Device, image_view: ImageView, allocator: *const ());
        pub fn vkCreateShaderModule(
            device: Device,
            create_info: *const ShaderModuleCreateInfo,
            allocator: *const (),
            shader_module: *mut ShaderModule,
        ) -> Result;
        pub fn vkDestroyShaderModule(
            device: Device,
            shader_module: ShaderModule,
            allocator: *const (),
        );
        pub fn vkCreatePipelineLayout(
            device: Device,
            create_info: *const PipelineLayoutCreateInfo,
            allocator: *const (),
            pipeline_layout: *mut PipelineLayout,
        ) -> Result;
        pub fn vkDestroyPipelineLayout(
            device: Device,
            pipeline_layout: PipelineLayout,
            allocator: *const (),
        );
        pub fn vkCreateRenderPass(
            device: Device,
            create_info: *const RenderPassCreateInfo,
            allocator: *const (),
            render_pass: *mut RenderPass,
        ) -> Result;
        pub fn vkDestroyRenderPass(
            device: Device,
            render_pass: RenderPass,
            allocator: *const (),
        );
        pub fn vkCreateGraphicsPipelines(
            device: Device,
            pipeline_cache: PipelineCache,
            create_info_count: u32,
            create_infos: *const GraphicsPipelineCreateInfo,
            allocator: *const (),
            pipelines: *mut Pipeline,
        ) -> Result;
        pub fn vkCreateComputePipelines(
            device: Device,
            pipeline_cache: PipelineCache,
            create_info_count: u32,
            create_infos: *const ComputePipelineCreateInfo,
            allocator: *const (),
            pipelines: *mut Pipeline,
        ) -> Result;
        pub fn vkDestroyPipeline(device: Device, pipeline: Pipeline, allocator: *const ());
        pub fn vkCreateFramebuffer(
            device: Device,
            create_info: *const FramebufferCreateInfo,
            allocator: *const (),
            framebuffer: *mut Framebuffer,
        ) -> Result;
        pub fn vkDestroyFramebuffer(
            device: Device,
            framebuffer: Framebuffer,
            allocator: *const (),
        );
        pub fn vkCreateCommandPool(
            device: Device,
            create_info: *const CommandPoolCreateInfo,
            allocator: *const (),
            command_pool: *mut CommandPool,
        ) -> Result;
        pub fn vkDestroyCommandPool(
            device: Device,
            command_pool: CommandPool,
            allocator: *const (),
        );
        pub fn vkCreateBuffer(
            device: Device,
            create_info: *const BufferCreateInfo,
            allocator: *const (),
            buffer: *mut Buffer,
        ) -> Result;
        pub fn vkDestroyBuffer(device: Device, buffer: Buffer, allocator: *const ());
        pub fn vkAllocateMemory(
            device: Device,
            allocate_info: *const MemoryAllocateInfo,
            allocator: *const (),
            memory: *mut DeviceMemory,
        ) -> Result;
        pub fn vkFreeMemory(device: Device, memory: DeviceMemory, allocator: *const ());
        pub fn vkBindBufferMemory(
            device: Device,
            buffer: Buffer,
            memory: DeviceMemory,
            memory_offset: DeviceSize,
        ) -> Result;
        pub fn vkAllocateCommandBuffers(
            device: Device,
            allocate_info: *const CommandBufferAllocateInfo,
            command_buffers: *mut CommandBuffer,
        ) -> Result;
        pub fn vkBeginCommandBuffer(
            command_buffer: CommandBuffer,
            begin_info: *const CommandBufferBeginInfo,
        ) -> Result;
        pub fn vkEndCommandBuffer(command_buffer: CommandBuffer) -> Result;
        pub fn vkCmdBeginRenderPass(
            command_buffer: CommandBuffer,
            begin_info: *const RenderPassBeginInfo,
            contents: SubpassContents,
        );
        pub fn vkCmdEndRenderPass(command_buffer: CommandBuffer);
        pub fn vkCmdNextSubpass(command_buffer: CommandBuffer, contents: SubpassContents);
        pub fn vkCmdBindPipeline(
            command_buffer: CommandBuffer,
            bind_point: PipelineBindPoint,
            pipeline: Pipeline,
        );
        pub fn vkCmdDraw(
            command_buffer: CommandBuffer,
            vertex_count: u32,
            instance_count: u32,
            first_vertex: u32,
            first_instance: u32,
        );
        pub fn vkCmdDrawIndexed(
            command_buffer: CommandBuffer,
            index_count: u32,
            instance_count: u32,
            first_index: u32,
            vertex_offset: i32,
            first_instance: u32,
        );
        pub fn vkCmdDispatch(
            command_buffer: CommandBuffer,
            group_count_x: u32,
            group_count_y: u32,
            group_count_z: u32,
        );
        pub fn vkCmdBindVertexBuffers(
            command_buffer: CommandBuffer,
            first_binding: u32,
            binding_count: u32,
            buffers: *const Buffer,
            offsets: *const DeviceSize,
        );
        pub fn vkCmdBindIndexBuffer(
            command_buffer: CommandBuffer,
            buffer: Buffer,
            offset: DeviceSize,
            index_type: IndexType,
        );
        pub fn vkCmdCopyBuffer(
            command_buffer: CommandBuffer,
            src_buffer: Buffer,
            dst_buffer: Buffer,
            region_count: u32,
            regions: *const BufferCopy,
        );
        pub fn vkCmdCopyBufferToImage(
            command_buffer: CommandBuffer,
            src_buffer: Buffer,
            dst_image: Image,
            dst_image_layout: ImageLayout,
            region_count: u32,
            regions: *const BufferImageCopy,
        );
        pub fn vkCmdPipelineBarrier(
            command_buffer: CommandBuffer,
            src_stage_mask: Flags,
            dst_stage_mask: Flags,
            dependency_flags: Flags,
            memory_barrier_count: u32,
            memory_barriers: *const MemoryBarrier,
            buffer_memory_barrier_count: u32,
            buffer_memory_barriers: *const BufferMemoryBarrier,
            image_memory_barrier_count: u32,
            image_memory_barriers: *const ImageMemoryBarrier,
        );
        pub fn vkCreateFence(
            device: Device,
            create_info: *const FenceCreateInfo,
            allocator: *const (),
            fence: *mut Fence,
        ) -> Result;
        pub fn vkDestroyFence(device: Device, fence: Fence, allocator: *const ());
        pub fn vkCreateSemaphore(
            device: Device,
            create_info: *const SemaphoreCreateInfo,
            allocator: *const (),
            semaphore: *mut Semaphore,
        ) -> Result;
        pub fn vkDestroySemaphore(device: Device, semaphore: Semaphore, allocator: *const ());
        pub fn vkWaitForFences(
            device: Device,
            fence_count: u32,
            fence: *const Fence,
            wait_all: Bool,
            timeout: u64,
        ) -> Result;
        pub fn vkResetFences(device: Device, fence_count: u32, fence: *const Fence) -> Result;
        pub fn vkAcquireNextImageKHR(
            device: Device,
            swapchain: Swapchain,
            timeout: u64,
            semaphore: Semaphore,
            fence: Fence,
            image_index: *mut u32,
        ) -> Result;
        pub fn vkQueueSubmit(
            queue: Queue,
            submit_count: u32,
            submit_infos: *const SubmitInfo,
            fence: Fence,
        ) -> Result;
        pub fn vkQueuePresentKHR(queue: Queue, present_info: *const PresentInfo) -> Result;
        pub fn vkResetCommandBuffer(command_buffer: CommandBuffer, flags: Flags) -> Result;
        pub fn vkMapMemory(
            device: Device,
            memory: DeviceMemory,
            offset: DeviceSize,
            size: DeviceSize,
            flags: Flags,
            data: *mut *mut (),
        ) -> Result;
        pub fn vkUnmapMemory(device: Device, memory: DeviceMemory);
        pub fn vkCreateDescriptorSetLayout(
            device: Device,
            create_info: *const DescriptorSetLayoutCreateInfo,
            allocator: *const (),
            set_layout: *mut DescriptorSetLayout,
        ) -> Result;
        pub fn vkDestroyDescriptorSetLayout(
            device: Device,
            set_layout: DescriptorSetLayout,
            allocator: *const (),
        );
        pub fn vkCreateDescriptorPool(
            device: Device,
            create_info: *const DescriptorPoolCreateInfo,
            allocator: *const (),
            descriptor_pool: *mut DescriptorPool,
        ) -> Result;
        pub fn vkDestroyDescriptorPool(
            device: Device,
            descriptor_pool: DescriptorPool,
            allocator: *const (),
        );
        pub fn vkUpdateDescriptorSets(
            device: Device,
            write_count: u32,
            writes: *const WriteDescriptorSet,
            copy_count: u32,
            copies: *const CopyDescriptorSet,
        );
        pub fn vkAllocateDescriptorSets(
            device: Device,
            allocate_info: *const DescriptorSetAllocateInfo,
            descriptor_sets: *mut DescriptorSet,
        ) -> Result;
        pub fn vkCmdBindDescriptorSets(
            command_buffer: CommandBuffer,
            bind_point: PipelineBindPoint,
            layout: PipelineLayout,
            first_set: u32,
            descriptor_set_count: u32,
            descriptor_sets: *const DescriptorSet,
            dynamic_offset_count: u32,
            dynamic_offsets: *const u32,
        );
        pub fn vkCreateImage(
            device: Device,
            create_info: *const ImageCreateInfo,
            allocator: *const (),
            image: *mut Image,
        ) -> Result;
        pub fn vkDestroyImage(device: Device, image: Image, allocator: *const ());
        pub fn vkGetImageMemoryRequirements(
            device: Device,
            image: Image,
            memory_requirements: *mut MemoryRequirements,
        );
        pub fn vkBindImageMemory(
            device: Device,
            image: Image,
            memory: DeviceMemory,
            memory_offset: DeviceSize,
        ) -> Result;
        pub fn vkCreateSampler(
            device: Device,
            create_info: *const SamplerCreateInfo,
            allocator: *const (),
            sampler: *mut Sampler,
        ) -> Result;
        pub fn vkDestroySampler(device: Device, sampler: Sampler, allocator: *const ());
    }
}

pub const KHR_SURFACE: &str = "VK_KHR_surface";
pub const KHR_XLIB_SURFACE: &str = "VK_KHR_xlib_surface";
pub const KHR_WIN32_SURFACE: &str = "VK_KHR_win32_surface";
pub const KHR_SWAPCHAIN: &str = "VK_KHR_swapchain";

pub const EXT_DEBUG_REPORT: &str = "VK_EXT_debug_report";
pub const EXT_DEBUG_UTILS: &str = "VK_EXT_debug_utils";

pub const LAYER_KHRONOS_VALIDATION: &str = "VK_LAYER_KHRONOS_validation";
pub const LAYER_LUNARG_STANDARD_VALIDATION: &str = "VK_LAYER_LUNARG_standard_validation";

pub const DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE: u32 = 0x00000001;
pub const DEBUG_UTILS_MESSAGE_SEVERITY_INFO: u32 = 0x00000010;
pub const DEBUG_UTILS_MESSAGE_SEVERITY_WARNING: u32 = 0x00000100;
pub const DEBUG_UTILS_MESSAGE_SEVERITY_ERROR: u32 = 0x00001000;

pub const DEBUG_UTILS_MESSAGE_TYPE_GENERAL: u32 = 0x00000001;
pub const DEBUG_UTILS_MESSAGE_TYPE_VALIDATION: u32 = 0x00000002;
pub const DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE: u32 = 0x00000004;

pub const QUEUE_GRAPHICS: u32 = 0x00000001;
pub const QUEUE_COMPUTE: u32 = 0x00000002;

pub const QUEUE_FAMILY_IGNORED: u32 = u32::MAX;

pub const IMAGE_ASPECT_COLOR: u32 = 0x00000001;
pub const IMAGE_ASPECT_DEPTH: u32 = 0x00000002;

pub const CULL_MODE_NONE: u32 = 0;
pub const CULL_MODE_FRONT: u32 = 0x00000001;
pub const CULL_MODE_BACK: u32 = 0x00000002;
pub const CULL_MODE_FRONT_AND_BACK: u32 = 0x00000003;

pub const COLOR_COMPONENT_R: u32 = 0x00000001;
pub const COLOR_COMPONENT_G: u32 = 0x00000002;
pub const COLOR_COMPONENT_B: u32 = 0x00000004;
pub const COLOR_COMPONENT_A: u32 = 0x00000008;

pub const SAMPLE_COUNT_1: u32 = 0x00000001;

pub const SUBPASS_EXTERNAL: u32 = u32::MAX;

pub const PIPELINE_STAGE_TOP_OF_PIPE: u32 = 0x00000001;
pub const PIPELINE_STAGE_FRAGMENT_SHADER: u32 = 0x00000080;
pub const PIPELINE_STAGE_EARLY_FRAGMENT_TESTS: u32 = 0x00000100;
pub const PIPELINE_STAGE_LATE_FRAGMENT_TESTS: u32 = 0x00000200;
pub const PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT: u32 = 0x00000400;
pub const PIPELINE_STAGE_COMPUTE_SHADER: u32 = 0x00000800;
pub const PIPELINE_STAGE_TRANSFER: u32 = 0x00001000;
pub const PIPELINE_STAGE_BOTTOM_OF_PIPE: u32 = 0x00002000;

pub const ACCESS_SHADER_READ: u32 = 0x00000020;
pub const ACCESS_COLOR_ATTACHMENT_READ: u32 = 0x00000080;
pub const ACCESS_COLOR_ATTACHMENT_WRITE: u32 = 0x00000100;
pub const ACCESS_DEPTH_STENCIL_ATTACHMENT_READ: u32 = 0x00000200;
pub const ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE: u32 = 0x00000400;

pub const BUFFER_USAGE_TRANSFER_SRC: u32 = 0x00000001;
pub const BUFFER_USAGE_TRANSFER_DST: u32 = 0x00000002;
pub const BUFFER_USAGE_UNIFORM: u32 = 0x00000010;
pub const BUFFER_USAGE_STORAGE: u32 = 0x00000020;
pub const BUFFER_USAGE_INDEX: u32 = 0x00000040;
pub const BUFFER_USAGE_VERTEX: u32 = 0x00000080;

pub const IMAGE_USAGE_TRANSFER_SRC: u32 = 0x00000001;
pub const IMAGE_USAGE_TRANSFER_DST: u32 = 0x00000002;
pub const IMAGE_USAGE_SAMPLED: u32 = 0x00000004;
pub const IMAGE_USAGE_STORAGE: u32 = 0x00000008;
pub const IMAGE_USAGE_COLOR_ATTACHMENT: u32 = 0x00000010;
pub const IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT: u32 = 0x00000020;
pub const IMAGE_USAGE_TRANSIENT_ATTACHMENT: u32 = 0x00000040;
pub const IMAGE_USAGE_INPUT_ATTACHMENT: u32 = 0x00000080;

pub const MEMORY_PROPERTY_DEVICE_LOCAL: u32 = 0x00000001;
pub const MEMORY_PROPERTY_HOST_VISIBLE: u32 = 0x00000002;
pub const MEMORY_PROPERTY_HOST_COHERENT: u32 = 0x00000004;
pub const MEMORY_PROPERTY_HOST_CACHED: u32 = 0x00000008;
pub const MEMORY_PROPERTY_LAZILY_ALLOCATED: u32 = 0x00000010;

pub const SHADER_STAGE_VERTEX: u32 = 0x00000001;
pub const SHADER_STAGE_FRAGMENT: u32 = 0x00000010;
pub const SHADER_STAGE_COMPUTE: u32 = 0x00000020;

pub type DebugUtilsMessengerCallback = fn(&DebugUtilsMessengerCallbackData) -> bool;

#[derive(Clone, Copy, Debug)]
pub enum Error {
    OutOfHostMemory,
    OutOfDeviceMemory,
    InitializationFailed,
    DeviceLost,
    MemoryMapFailed,
    LayerNotPresent,
    ExtensionNotPresent,
    FeatureNotPresent,
    IncompatibleDriver,
    TooManyObjects,
    FormatNotSupported,
    FragmentedPool,
    Unknown,
    OutOfPoolMemory,
    Fragmentation,
    InvalidExternalHandle,
    SurfaceLost,
    NativeWindowInUse,
    Suboptimal,
    OutOfDate,
    InvalidShader,
    FullScreenExclusiveModeLost,
    InvalidOpaqueCaptureAddress,
    CompressionExhausted,
}

#[derive(Clone, Copy, Debug)]
pub enum Format {
    Rgba8Srgb,
    Bgra8Srgb,
    R16Uint,
    R32Uint,
    R32Sfloat,
    Rg32Sfloat,
    Rgb32Uint,
    Rgb32Sfloat,
    Rgba32Sfloat,
    D32Sfloat,
}

impl Format {
    pub fn aspect_mask(self) -> u32 {
        match self {
            Self::D32Sfloat => IMAGE_ASPECT_DEPTH,
            _ => IMAGE_ASPECT_COLOR,
        }
    }
}

#[derive(Clone, Copy)]
pub enum IndexType {
    Uint16,
    Uint32,
}

#[derive(Clone, Copy)]
pub enum ColorSpace {
    SrgbNonlinear,
}

#[derive(Clone, Copy)]
pub enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
    FifoRelaxed,
}

pub type Extent2d = (u32, u32);
pub type Extent3d = (u32, u32, u32);

pub type Offset2d = (i32, i32);
pub type Offset3d = (i32, i32, i32);

#[derive(Clone, Copy)]
pub struct SurfaceCapabilities {
    pub min_image_count: u32,
    pub max_image_count: u32,
    pub current_extent: Extent2d,
    pub min_image_extent: Extent2d,
    pub max_image_extent: Extent2d,
    pub max_image_array_layers: u32,
    pub supported_transforms: u32,
    pub current_transform: u32,
    pub supported_composite_alpha: u32,
    pub supported_usage_flags: u32,
}

#[derive(Clone, Copy)]
pub struct SurfaceFormat {
    pub format: Format,
    pub color_space: ColorSpace,
}

#[derive(Clone, Copy)]
pub enum SharingMode {
    Exclusive,
}

#[derive(Clone, Copy)]
pub enum CompositeAlpha {
    Opaque,
}

#[derive(Clone, Copy)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl From<(u32, u32, u32)> for Version {
    fn from(tuple: (u32, u32, u32)) -> Self {
        Version {
            major: tuple.0,
            minor: tuple.1,
            patch: tuple.2,
        }
    }
}

impl From<Version> for u32 {
    fn from(version: Version) -> Self {
        (version.major << 22 | version.minor << 12 | version.patch) as u32
    }
}

#[derive(Clone, Copy)]
pub struct ApplicationInfo<'a> {
    pub application_name: &'a str,
    pub application_version: Version,
    pub engine_name: &'a str,
    pub engine_version: Version,
    pub api_version: Version,
}

#[derive(Clone, Copy)]
pub struct InstanceCreateInfo<'a> {
    pub application_info: &'a ApplicationInfo<'a>,
    pub extensions: &'a [&'a str],
    pub layers: &'a [&'a str],
    pub debug_utils: &'a Option<DebugUtilsMessengerCreateInfo>,
}

#[derive(Clone, Copy)]
pub struct DebugUtilsMessengerCreateInfo {
    pub message_severity: u32,
    pub message_type: u32,
    pub user_callback: DebugUtilsMessengerCallback,
}

#[derive(Clone, Copy)]
pub struct DebugUtilsMessengerCallbackData<'a> {
    pub message_severity: u32,
    pub message_type: u32,
    pub message: &'a str,
}

pub struct Instance {
    handle: ffi::Instance,
}

impl Instance {
    pub fn new(create_info: InstanceCreateInfo<'_>) -> Result<Rc<Instance>, Error> {
        let application_name = CString::new(create_info.application_info.application_name).unwrap();

        let application_version: u32 = create_info.application_info.application_version.into();

        let engine_name = CString::new(create_info.application_info.application_name).unwrap();

        let engine_version: u32 = create_info.application_info.engine_version.into();

        let api_version: u32 = create_info.application_info.api_version.into();

        let application_info = ffi::ApplicationInfo {
            structure_type: ffi::StructureType::ApplicationInfo,
            p_next: ptr::null(),
            application_name: application_name.as_ptr(),
            application_version,
            engine_name: engine_name.as_ptr(),
            engine_version,
            api_version,
        };

        let layer_names = create_info
            .layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect::<Vec<_>>();

        let enabled_layer_names = layer_names
            .iter()
            .map(|string| string.as_ptr())
            .collect::<Vec<_>>();

        let enabled_layer_names_ptr = if layer_names.len() > 0 {
            enabled_layer_names.as_ptr()
        } else {
            ptr::null()
        };

        let extension_names = create_info
            .extensions
            .iter()
            .map(|extension_name| CString::new(*extension_name).unwrap())
            .collect::<Vec<_>>();

        let enabled_extension_names = extension_names
            .iter()
            .map(|string| string.as_ptr())
            .collect::<Vec<_>>();

        let enabled_extension_names_ptr = if extension_names.len() > 0 {
            enabled_extension_names.as_ptr()
        } else {
            ptr::null()
        };

        let debug_utils = if let Some(create_info) = create_info.debug_utils {
            let g = unsafe { mem::transmute(create_info.user_callback) };

            let create_info = ffi::DebugUtilsMessengerCreateInfo {
                structure_type: ffi::StructureType::DebugUtilsMessengerCreateInfo,
                p_next: ptr::null(),
                flags: 0,
                message_severity: create_info.message_severity as _,
                message_type: create_info.message_type as _,
                user_callback: ffi::debug_utils_messenger_callback,
                user_data: g,
            };

            Some(create_info)
        } else {
            None
        };

        let p_next = if let Some(create_info) = &debug_utils {
            unsafe { mem::transmute::<_, _>(create_info) }
        } else {
            ptr::null()
        };

        let create_info = ffi::InstanceCreateInfo {
            structure_type: ffi::StructureType::InstanceCreateInfo,
            p_next,
            flags: 0,
            application_info: &application_info,
            enabled_layer_count: create_info.layers.len() as _,
            enabled_layer_names: enabled_layer_names_ptr,
            enabled_extension_count: create_info.extensions.len() as _,
            enabled_extension_names: enabled_extension_names_ptr,
        };

        let mut handle = MaybeUninit::<ffi::Instance>::uninit();

        let result =
            unsafe { ffi::vkCreateInstance(&create_info, ptr::null(), handle.as_mut_ptr()) };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let instance = Self { handle };

                let instance = Rc::new(instance);

                Ok(instance)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::InitializationFailed => Err(Error::InitializationFailed),
            ffi::Result::LayerNotPresent => Err(Error::LayerNotPresent),
            ffi::Result::ExtensionNotPresent => Err(Error::ExtensionNotPresent),
            ffi::Result::IncompatibleDriver => Err(Error::IncompatibleDriver),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyInstance(self.handle, ptr::null()) };
    }
}

pub struct DebugUtilsMessenger {
    instance: Rc<Instance>,
    handle: ffi::DebugUtilsMessenger,
}

impl DebugUtilsMessenger {
    pub fn new(
        instance: Rc<Instance>,
        create_info: DebugUtilsMessengerCreateInfo,
    ) -> Result<Self, Error> {
        let f_name = CStr::from_bytes_with_nul(b"vkCreateDebugUtilsMessengerEXT\0").unwrap();

        let f = unsafe { ffi::vkGetInstanceProcAddr(instance.handle, f_name.as_ptr()) };

        if f == ptr::null() {
            return Err(Error::ExtensionNotPresent);
        }

        let f = unsafe { mem::transmute::<_, ffi::CreateDebugUtilsMessenger>(f) };

        let g = unsafe { mem::transmute(create_info.user_callback) };

        let create_info = ffi::DebugUtilsMessengerCreateInfo {
            structure_type: ffi::StructureType::DebugUtilsMessengerCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            message_severity: create_info.message_severity as _,
            message_type: create_info.message_type as _,
            user_callback: ffi::debug_utils_messenger_callback,
            user_data: g,
        };

        let mut handle = MaybeUninit::<ffi::DebugUtilsMessenger>::uninit();

        let result = unsafe {
            f(
                instance.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let debug_utils_messenger = Self { instance, handle };

                Ok(debug_utils_messenger)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for DebugUtilsMessenger {
    fn drop(&mut self) {
        let f_name = CStr::from_bytes_with_nul(b"vkDestroyDebugUtilsMessengerEXT\0").unwrap();

        let f = unsafe { ffi::vkGetInstanceProcAddr(self.instance.handle, f_name.as_ptr()) };

        if f == ptr::null() {
            panic!("extension not present, but handle already created");
        }

        let f = unsafe { mem::transmute::<_, ffi::DestroyDebugUtilsMessenger>(f) };

        unsafe { f(self.instance.handle, self.handle, ptr::null()) };
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum PhysicalDeviceType {
    Other,
    Integrated,
    Discrete,
    Virtual,
    Cpu,
}

pub struct PhysicalDeviceLimits {
    pub max_image_dimension_2d: u32,
}

//TODO add more info
pub struct PhysicalDeviceProperties {
    pub device_type: PhysicalDeviceType,
    pub device_name: String,
    pub limits: PhysicalDeviceLimits,
}

#[derive(Clone, Copy, Default)]
pub struct PhysicalDeviceFeatures {
    pub robust_buffer_access: bool,
    pub full_draw_index_uint_32: bool,
    pub image_cube_array: bool,
    pub independent_blend: bool,
    pub geometry_shader: bool,
    pub tessellation_shader: bool,
    pub sample_rate_shading: bool,
    pub dual_src_blend: bool,
    pub logic_op: bool,
    pub multi_draw_indirect: bool,
    pub draw_indirect_first_instance: bool,
    pub depth_clamp: bool,
    pub depth_bias_clamp: bool,
    pub fill_mode_non_solid: bool,
    pub depth_bounds: bool,
    pub wide_lines: bool,
    pub large_points: bool,
    pub alpha_to_one: bool,
    pub multi_viewport: bool,
    pub sampler_anisotropy: bool,
    pub texture_compression_etc_2: bool,
    pub texture_compression_astc_ldr: bool,
    pub texture_compression_bc: bool,
    pub occlusion_query_precise: bool,
    pub pipeline_statistics_query: bool,
    pub vertex_pipeline_stores_and_atomics: bool,
    pub fragment_stores_and_atomics: bool,
    pub shader_tessellation_and_geometry_point_size: bool,
    pub shader_image_gather_extended: bool,
    pub shader_storage_image_extended_formats: bool,
    pub shader_storage_image_multisample: bool,
    pub shader_storage_image_read_without_format: bool,
    pub shader_storage_image_write_without_format: bool,
    pub shader_uniform_buffer_array_dynamic_indexing: bool,
    pub shader_sampled_image_array_dynamic_indexing: bool,
    pub shader_storage_buffer_array_dynamic_indexing: bool,
    pub shader_storage_image_array_dynamic_indexing: bool,
    pub shader_clip_distance: bool,
    pub shader_cull_distance: bool,
    pub shader_float_64: bool,
    pub shader_int_64: bool,
    pub shader_int_16: bool,
    pub shader_resource_residency: bool,
    pub shader_resource_min_lod: bool,
    pub sparse_binding: bool,
    pub sparse_residency_buffer: bool,
    pub sparse_residency_image_2_d: bool,
    pub sparse_residency_image_3_d: bool,
    pub sparse_residency_2_samples: bool,
    pub sparse_residency_4_samples: bool,
    pub sparse_residency_8_samples: bool,
    pub sparse_residency_16_samples: bool,
    pub sparse_residency_aliased: bool,
    pub variable_multisample_rate: bool,
    pub inherited_queries: bool,
}

pub struct PhysicalDevice {
    handle: ffi::PhysicalDevice,
}

impl PhysicalDevice {
    pub fn enumerate(instance: Rc<Instance>) -> Vec<Rc<Self>> {
        let mut handle_count: u32 = 0;

        unsafe {
            ffi::vkEnumeratePhysicalDevices(instance.handle, &mut handle_count, ptr::null_mut())
        };

        let mut handles = Vec::<ffi::PhysicalDevice>::with_capacity(handle_count as _);

        unsafe {
            ffi::vkEnumeratePhysicalDevices(
                instance.handle,
                &mut handle_count,
                handles.as_mut_ptr(),
            )
        };

        unsafe { handles.set_len(handle_count as _) };

        let physical_devices = handles
            .into_iter()
            .map(|handle| Self { handle })
            .map(|physical_device| Rc::new(physical_device))
            .collect::<Vec<_>>();

        physical_devices
    }

    pub fn properties(&self) -> PhysicalDeviceProperties {
        let mut properties = MaybeUninit::<ffi::PhysicalDeviceProperties>::uninit();

        unsafe { ffi::vkGetPhysicalDeviceProperties(self.handle, properties.as_mut_ptr()) };

        let properties = unsafe { properties.assume_init() };

        let device_type = match properties.device_type {
            ffi::PhysicalDeviceType::Other => PhysicalDeviceType::Other,
            ffi::PhysicalDeviceType::Integrated => PhysicalDeviceType::Integrated,
            ffi::PhysicalDeviceType::Discrete => PhysicalDeviceType::Discrete,
            ffi::PhysicalDeviceType::Virtual => PhysicalDeviceType::Virtual,
            ffi::PhysicalDeviceType::Cpu => PhysicalDeviceType::Cpu,
        };

        let device_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
            .to_str()
            .unwrap()
            .to_owned();

        let limits = PhysicalDeviceLimits {
            max_image_dimension_2d: properties.limits.max_image_dimension_2d,
        };

        PhysicalDeviceProperties {
            device_type,
            device_name,
            limits,
        }
    }

    //TODO
    pub fn features(&self) -> PhysicalDeviceFeatures {
        unimplemented!();
    }

    pub fn queue_families(&self) -> Vec<QueueFamilyProperties> {
        let mut queue_family_count: u32 = 0;

        unsafe {
            ffi::vkGetPhysicalDeviceQueueFamilyProperties(
                self.handle,
                &mut queue_family_count,
                ptr::null_mut(),
            )
        };

        let mut queue_families =
            Vec::<ffi::QueueFamilyProperties>::with_capacity(queue_family_count as _);

        unsafe {
            ffi::vkGetPhysicalDeviceQueueFamilyProperties(
                self.handle,
                &mut queue_family_count,
                queue_families.as_mut_ptr(),
            )
        };

        unsafe { queue_families.set_len(queue_family_count as _) };

        let queue_families = queue_families
            .into_iter()
            .map(|queue_family| QueueFamilyProperties {
                queue_flags: queue_family.queue_flags,
                queue_count: queue_family.queue_count,
            })
            .collect::<Vec<_>>();

        queue_families
    }

    pub fn surface_format(&self, surface: &Surface) -> SurfaceFormat {
        SurfaceFormat {
            format: Format::Bgra8Srgb,
            color_space: ColorSpace::SrgbNonlinear,
        }
    }

    //TODO
    pub fn surface_capabilities(&self, surface: &Surface) -> SurfaceCapabilities {
        let mut surface_capabilities = MaybeUninit::<ffi::SurfaceCapabilities>::uninit();

        unsafe {
            ffi::vkGetPhysicalDeviceSurfaceCapabilitiesKHR(
                self.handle,
                surface.handle,
                surface_capabilities.as_mut_ptr(),
            )
        };

        let surface_capabilities = unsafe { surface_capabilities.assume_init() };

        let current_extent = (
            surface_capabilities.current_extent[0],
            surface_capabilities.current_extent[1],
        );

        let min_image_extent = (
            surface_capabilities.min_image_extent[0],
            surface_capabilities.min_image_extent[1],
        );

        let max_image_extent = (
            surface_capabilities.max_image_extent[0],
            surface_capabilities.max_image_extent[1],
        );

        SurfaceCapabilities {
            min_image_count: surface_capabilities.min_image_count,
            max_image_count: surface_capabilities.max_image_count,
            current_extent,
            min_image_extent,
            max_image_extent,
            max_image_array_layers: surface_capabilities.max_image_array_layers,
            supported_transforms: surface_capabilities.supported_transforms,
            current_transform: surface_capabilities.current_transform,
            supported_composite_alpha: surface_capabilities.supported_composite_alpha,
            supported_usage_flags: surface_capabilities.supported_usage_flags,
        }
    }

    pub fn surface_supported(
        &self,
        surface: &Surface,
        queue_family_index: u32,
    ) -> Result<bool, Error> {
        let mut supported = MaybeUninit::<ffi::Bool>::uninit();

        let result = unsafe {
            ffi::vkGetPhysicalDeviceSurfaceSupportKHR(
                self.handle,
                queue_family_index,
                surface.handle,
                supported.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let supported = unsafe { supported.assume_init() };

                let supported = supported != 0;

                Ok(supported)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::SurfaceLost => Err(Error::SurfaceLost),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn memory_properties(&self) -> MemoryProperties {
        let mut memory_properties = MaybeUninit::<ffi::PhysicalDeviceMemoryProperties>::uninit();

        unsafe {
            ffi::vkGetPhysicalDeviceMemoryProperties(self.handle, memory_properties.as_mut_ptr())
        };

        let memory_properties = unsafe { memory_properties.assume_init() };

        let memory_types = (0..memory_properties.memory_type_count)
            .into_iter()
            .map(|i| memory_properties.memory_types[i as usize])
            .map(|t| MemoryType {
                property_flags: t.property_flags,
                heap_index: t.heap_index,
            })
            .collect::<Vec<_>>();

        let memory_heaps = (0..memory_properties.memory_heap_count)
            .into_iter()
            .map(|i| memory_properties.memory_heaps[i as usize])
            .map(|h| MemoryHeap {
                size: h.size,
                flags: h.flags,
            })
            .collect::<Vec<_>>();

        MemoryProperties {
            memory_types,
            memory_heaps,
        }
    }

    //TODO
    pub fn surface_formats(&self, surface: &Surface) -> Vec<SurfaceFormat> {
        unimplemented!();
    }

    //TODO
    pub fn surface_present_modes(&self, surface: &Surface) -> Vec<PresentMode> {
        unimplemented!();
    }
}

pub struct QueueFamilyProperties {
    pub queue_flags: u32,
    pub queue_count: u32,
}

pub struct DeviceQueueCreateInfo<'a> {
    pub queue_family_index: u32,
    pub queue_priorities: &'a [f32],
}

pub struct DeviceCreateInfo<'a> {
    pub queues: &'a [DeviceQueueCreateInfo<'a>],
    pub enabled_features: &'a PhysicalDeviceFeatures,
    pub extensions: &'a [&'a str],
    pub layers: &'a [&'a str],
}

pub struct Device {
    handle: ffi::Device,
}

impl Device {
    pub fn new(
        physical_device: &PhysicalDevice,
        create_info: DeviceCreateInfo<'_>,
    ) -> Result<Rc<Device>, Error> {
        let queue_create_infos = create_info
            .queues
            .iter()
            .map(|create_info| ffi::DeviceQueueCreateInfo {
                structure_type: ffi::StructureType::DeviceQueueCreateInfo,
                p_next: ptr::null(),
                flags: 0,
                queue_family_index: create_info.queue_family_index,
                queue_count: create_info.queue_priorities.len() as _,
                queue_priorities: create_info.queue_priorities.as_ptr(),
            })
            .collect::<Vec<_>>();

        let layer_names = create_info
            .layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect::<Vec<_>>();

        let enabled_layer_names = layer_names
            .iter()
            .map(|string| string.as_ptr())
            .collect::<Vec<_>>();

        let extension_names = create_info
            .extensions
            .iter()
            .map(|extension_name| CString::new(*extension_name).unwrap())
            .collect::<Vec<_>>();

        let enabled_extension_names = extension_names
            .iter()
            .map(|string| string.as_ptr())
            .collect::<Vec<_>>();

        let enabled_features = create_info.enabled_features.clone().into();

        let create_info = ffi::DeviceCreateInfo {
            structure_type: ffi::StructureType::DeviceCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            queue_create_info_count: queue_create_infos.len() as _,
            queue_create_infos: queue_create_infos.as_ptr(),
            enabled_layer_count: create_info.layers.len() as _,
            enabled_layer_names: enabled_layer_names.as_ptr(),
            enabled_extension_count: create_info.extensions.len() as _,
            enabled_extension_names: enabled_extension_names.as_ptr(),
            enabled_features: &enabled_features,
        };

        let mut handle = MaybeUninit::<ffi::Device>::uninit();

        let result = unsafe {
            ffi::vkCreateDevice(
                physical_device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let device = Self { handle };

                let device = Rc::new(device);

                Ok(device)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::InitializationFailed => Err(Error::InitializationFailed),
            ffi::Result::ExtensionNotPresent => Err(Error::ExtensionNotPresent),
            ffi::Result::FeatureNotPresent => Err(Error::FeatureNotPresent),
            ffi::Result::TooManyObjects => Err(Error::TooManyObjects),
            ffi::Result::DeviceLost => Err(Error::DeviceLost),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn queue(&self, queue_family_index: u32) -> Queue {
        let mut handle = MaybeUninit::<ffi::Queue>::uninit();

        unsafe {
            ffi::vkGetDeviceQueue(self.handle, queue_family_index as _, 0, handle.as_mut_ptr())
        };

        let handle = unsafe { handle.assume_init() };

        Queue { handle }
    }

    pub fn wait_idle(&self) -> Result<(), Error> {
        let result = unsafe { ffi::vkDeviceWaitIdle(self.handle) };

        match result {
            ffi::Result::Success => Ok(()),
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::DeviceLost => Err(Error::DeviceLost),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyDevice(self.handle, ptr::null()) };
    }
}

pub struct Queue {
    handle: ffi::Queue,
}

impl Queue {
    pub fn submit(
        &mut self,
        submit_infos: &'_ [SubmitInfo],
        fence: Option<&'_ mut Fence>,
    ) -> Result<(), Error> {
        let wait_semaphores = submit_infos
            .iter()
            .map(|submit_info| {
                submit_info
                    .wait_semaphores
                    .iter()
                    .map(|semaphore| semaphore.handle)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let command_buffers = submit_infos
            .iter()
            .map(|submit_info| {
                submit_info
                    .command_buffers
                    .iter()
                    .map(|command_buffer| command_buffer.handle)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let signal_semaphores = submit_infos
            .iter()
            .map(|submit_info| {
                submit_info
                    .signal_semaphores
                    .iter()
                    .map(|semaphore| semaphore.handle)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let submit_infos = submit_infos
            .iter()
            .enumerate()
            .map(|(i, submit_info)| ffi::SubmitInfo {
                structure_type: ffi::StructureType::SubmitInfo,
                p_next: ptr::null(),
                wait_semaphore_count: wait_semaphores[i].len() as _,
                wait_semaphores: wait_semaphores[i].as_ptr(),
                wait_dst_stage_mask: submit_info.wait_stages.as_ptr() as _,
                command_buffer_count: command_buffers[i].len() as _,
                command_buffers: command_buffers[i].as_ptr(),
                signal_semaphore_count: signal_semaphores[i].len() as _,
                signal_semaphores: signal_semaphores[i].as_ptr(),
            })
            .collect::<Vec<_>>();

        let fence = fence.map_or(ffi::Fence::null(), |fence| fence.handle);

        let result = unsafe {
            ffi::vkQueueSubmit(
                self.handle,
                submit_infos.len() as _,
                submit_infos.as_ptr(),
                fence,
            )
        };

        match result {
            ffi::Result::Success => Ok(()),
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::DeviceLost => Err(Error::DeviceLost),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn present(&mut self, present_info: PresentInfo) -> Result<(), Error> {
        let wait_semaphores = present_info
            .wait_semaphores
            .iter()
            .map(|semaphore| semaphore.handle)
            .collect::<Vec<_>>();

        let swapchains = present_info
            .swapchains
            .iter()
            .map(|swapchain| swapchain.handle)
            .collect::<Vec<_>>();

        let present_info = ffi::PresentInfo {
            structure_type: ffi::StructureType::PresentInfo,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as _,
            wait_semaphores: wait_semaphores.as_ptr(),
            swapchain_count: swapchains.len() as _,
            swapchains: swapchains.as_ptr(),
            image_indices: present_info.image_indices.as_ptr() as _,
            //TODO
            results: ptr::null(),
        };

        let result = unsafe { ffi::vkQueuePresentKHR(self.handle, &present_info) };

        match result {
            ffi::Result::Success => Ok(()),
            ffi::Result::Suboptimal => Err(Error::Suboptimal),
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::DeviceLost => Err(Error::DeviceLost),
            ffi::Result::OutOfDate => Err(Error::OutOfDate),
            ffi::Result::SurfaceLost => Err(Error::SurfaceLost),
            ffi::Result::FullScreenExclusiveModeLost => Err(Error::FullScreenExclusiveModeLost),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn wait_idle(&self) -> Result<(), Error> {
        let result = unsafe { ffi::vkQueueWaitIdle(self.handle) };

        match result {
            ffi::Result::Success => Ok(()),
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::DeviceLost => Err(Error::DeviceLost),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

pub struct Surface {
    instance: Rc<Instance>,
    handle: ffi::Surface,
}

#[cfg(target_os = "windows")]
impl Surface {
    pub fn new(instance: Rc<Instance>, window: &impl HasRawWindowHandle) -> Rc<Self> {
        match window.raw_window_handle() {
            RawWindowHandle::Win32(win32_handle) => {
                let create_info = ffi::Win32SurfaceCreateInfo {
                    structure_type: ffi::StructureType::Win32SurfaceCreateInfo,
                    p_next: ptr::null(),
                    flags: 0,
                    hinstance: unsafe { mem::transmute(win32_handle.hinstance) } ,
                    hwnd: unsafe { mem::transmute(win32_handle.hwnd) },
                };

                let mut handle = MaybeUninit::<ffi::Surface>::uninit();

                unsafe {
                    ffi::vkCreateWin32SurfaceKHR(
                        instance.handle,
                        &create_info,
                        ptr::null(),
                        handle.as_mut_ptr(),
                    )
                };

                let handle = unsafe { handle.assume_init() };

                Rc::new(Self { instance, handle })
            }
            RawWindowHandle::Xcb(_) => unimplemented!("xcb unimplemented"),
            RawWindowHandle::Wayland(_) => unimplemented!("wayland unimplemented"),
            _ => panic!("unsupported window handle"),
        }
    }
}

#[cfg(target_os = "linux")]
impl Surface {
    pub fn new(instance: Rc<Instance>, window: &impl HasRawWindowHandle) -> Rc<Self> {
        match window.raw_window_handle() {
            RawWindowHandle::Xlib(xlib_handle) => {
                let create_info = ffi::XlibSurfaceCreateInfo {
                    structure_type: ffi::StructureType::XlibSurfaceCreateInfo,
                    p_next: ptr::null(),
                    flags: 0,
                    display: xlib_handle.display,
                    window: xlib_handle.window,
                };

                let mut handle = MaybeUninit::<ffi::Surface>::uninit();

                unsafe {
                    ffi::vkCreateXlibSurfaceKHR(
                        instance.handle,
                        &create_info,
                        ptr::null(),
                        handle.as_mut_ptr(),
                    )
                };

                let handle = unsafe { handle.assume_init() };

                Rc::new(Self { instance, handle })
            }
            RawWindowHandle::Xcb(_) => unimplemented!("xcb unimplemented"),
            RawWindowHandle::Wayland(_) => unimplemented!("wayland unimplemented"),
            _ => panic!("unsupported window handle"),
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroySurfaceKHR(self.instance.handle, self.handle, ptr::null()) };
    }
}

pub struct SwapchainCreateInfo<'a> {
    pub surface: &'a Surface,
    pub min_image_count: u32,
    pub image_format: Format,
    pub image_color_space: ColorSpace,
    pub image_extent: Extent2d,
    pub image_array_layers: u32,
    pub image_usage: u32,
    pub image_sharing_mode: SharingMode,
    pub queue_family_indices: &'a [u32],
    pub pre_transform: u32,
    pub composite_alpha: CompositeAlpha,
    pub present_mode: PresentMode,
    pub clipped: bool,
    pub old_swapchain: Option<Swapchain>,
}

pub struct Swapchain {
    device: Rc<Device>,
    handle: ffi::Swapchain,
}

impl Swapchain {
    pub fn new(device: Rc<Device>, create_info: SwapchainCreateInfo<'_>) -> Result<Self, Error> {
        let image_format = match create_info.image_format {
            Format::Bgra8Srgb => ffi::Format::Bgra8Srgb,
            _ => unimplemented!(),
        };

        let image_color_space = match create_info.image_color_space {
            ColorSpace::SrgbNonlinear => ffi::ColorSpace::SrgbNonlinear,
            _ => unimplemented!(),
        };

        let image_extent = [
            create_info.image_extent.0 as _,
            create_info.image_extent.1 as _,
        ];

        let image_usage = create_info.image_usage;

        let image_sharing_mode = match create_info.image_sharing_mode {
            SharingMode::Exclusive => ffi::SharingMode::Exclusive,
            _ => unimplemented!(),
        };

        let queue_family_indices = unsafe { mem::transmute(&create_info.queue_family_indices) };

        let composite_alpha = match create_info.composite_alpha {
            CompositeAlpha::Opaque => ffi::CompositeAlpha::Opaque,
            _ => unimplemented!(),
        };

        let present_mode = match create_info.present_mode {
            PresentMode::Immediate => ffi::PresentMode::Immediate,
            PresentMode::Mailbox => ffi::PresentMode::Mailbox,
            PresentMode::Fifo => ffi::PresentMode::Fifo,
            PresentMode::FifoRelaxed => ffi::PresentMode::FifoRelaxed,
        };

        //if this is not extracted, and handle is not declared separately,
        //then swapchain will be dropped too early.
        let old_swapchain = create_info.old_swapchain;

        let old_swapchain_handle = old_swapchain
            .as_ref()
            .map_or(ffi::Swapchain::null(), |swapchain| swapchain.handle);

        let create_info = ffi::SwapchainCreateInfo {
            structure_type: ffi::StructureType::SwapchainCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            surface: create_info.surface.handle,
            min_image_count: create_info.min_image_count,
            image_format,
            image_color_space,
            image_extent,
            image_array_layers: create_info.image_array_layers,
            image_usage,
            image_sharing_mode,
            queue_family_index_count: create_info.queue_family_indices.len() as _,
            queue_family_indices,
            pre_transform: create_info.pre_transform,
            composite_alpha,
            present_mode,
            clipped: create_info.clipped as _,
            old_swapchain: old_swapchain_handle,
        };

        let mut handle = MaybeUninit::<ffi::Swapchain>::uninit();

        let result = unsafe {
            ffi::vkCreateSwapchainKHR(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let swapchain = Self { device, handle };

                Ok(swapchain)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::DeviceLost => Err(Error::DeviceLost),
            ffi::Result::SurfaceLost => Err(Error::SurfaceLost),
            ffi::Result::NativeWindowInUse => Err(Error::NativeWindowInUse),
            ffi::Result::InitializationFailed => Err(Error::InitializationFailed),
            ffi::Result::CompressionExhausted => Err(Error::CompressionExhausted),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn images(&self) -> Vec<Image> {
        let mut swapchain_image_count: u32 = 0;

        unsafe {
            ffi::vkGetSwapchainImagesKHR(
                self.device.handle,
                self.handle,
                &mut swapchain_image_count,
                ptr::null_mut(),
            )
        };

        let mut swapchain_images = Vec::<ffi::Image>::with_capacity(swapchain_image_count as _);

        unsafe {
            ffi::vkGetSwapchainImagesKHR(
                self.device.handle,
                self.handle,
                &mut swapchain_image_count,
                swapchain_images.as_mut_ptr(),
            )
        };

        unsafe { swapchain_images.set_len(swapchain_image_count as _) };

        let swapchain_images = swapchain_images
            .into_iter()
            .map(|handle| Image {
                device: self.device.clone(),
                handle,
                user: false,
            })
            .collect::<Vec<_>>();

        swapchain_images
    }

    pub fn acquire_next_image(
        &mut self,
        timeout: u64,
        semaphore: Option<&'_ mut Semaphore>,
        fence: Option<&'_ mut Fence>,
    ) -> Result<u32, Error> {
        let mut image_index = 0;

        let semaphore = semaphore.map_or(ffi::Semaphore::null(), |semaphore| semaphore.handle);

        let fence = fence.map_or(ffi::Fence::null(), |fence| fence.handle);

        let result = unsafe {
            ffi::vkAcquireNextImageKHR(
                self.device.handle,
                self.handle,
                timeout,
                semaphore,
                fence,
                &mut image_index,
            )
        };
        //TODO this might be wrong
        match result {
            ffi::Result::Success | ffi::Result::Timeout | ffi::Result::NotReady => Ok(image_index),
            ffi::Result::Suboptimal => Err(Error::Suboptimal),
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::DeviceLost => Err(Error::DeviceLost),
            ffi::Result::OutOfDate => Err(Error::OutOfDate),
            ffi::Result::SurfaceLost => Err(Error::SurfaceLost),
            ffi::Result::FullScreenExclusiveModeLost => Err(Error::FullScreenExclusiveModeLost),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroySwapchainKHR(self.device.handle, self.handle, ptr::null()) };
    }
}

pub enum ImageType {
    OneDim,
    TwoDim,
    ThreeDim,
}

pub enum ImageTiling {
    Optimal,
    Linear,
}

pub struct ImageCreateInfo {
    pub image_type: ImageType,
    pub format: Format,
    pub extent: Extent3d,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: u32,
    pub tiling: ImageTiling,
    pub image_usage: u32,
    pub initial_layout: ImageLayout,
}

pub struct Image {
    device: Rc<Device>,
    handle: ffi::Image,
    user: bool,
}

impl Image {
    pub fn new(device: Rc<Device>, create_info: ImageCreateInfo) -> Result<Self, Error> {
        let create_info = ffi::ImageCreateInfo {
            structure_type: ffi::StructureType::ImageCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            image_type: create_info.image_type.into(),
            extent: [
                create_info.extent.0,
                create_info.extent.1,
                create_info.extent.2,
            ],
            mip_levels: create_info.mip_levels,
            array_layers: create_info.array_layers,
            format: create_info.format.into(),
            tiling: create_info.tiling.into(),
            initial_layout: create_info.initial_layout.into(),
            image_usage: create_info.image_usage as _,
            samples: create_info.samples,
            sharing_mode: ffi::SharingMode::Exclusive,
            queue_family_index_count: 0,
            queue_family_indices: ptr::null(),
        };

        let mut handle = MaybeUninit::<ffi::Image>::uninit();

        let result = unsafe {
            ffi::vkCreateImage(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let image = Self {
                    device,
                    handle,
                    user: true,
                };

                Ok(image)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory)?,
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory)?,
            ffi::Result::CompressionExhausted => Err(Error::CompressionExhausted)?,
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn memory_requirements(&self) -> MemoryRequirements {
        let mut memory_requirements = MaybeUninit::<ffi::MemoryRequirements>::uninit();

        unsafe {
            ffi::vkGetImageMemoryRequirements(
                self.device.handle,
                self.handle,
                memory_requirements.as_mut_ptr(),
            )
        };

        let memory_requirements = unsafe { memory_requirements.assume_init() };

        MemoryRequirements {
            size: memory_requirements.size,
            alignment: memory_requirements.alignment,
            memory_type: memory_requirements.memory_type,
        }
    }

    pub fn bind_memory(&mut self, memory: &Memory) -> Result<(), Error> {
        if !self.user {
            panic!("cannot bind memory to non-user image");
        }

        let result =
            unsafe { ffi::vkBindImageMemory(self.device.handle, self.handle, memory.handle, 0) };

        match result {
            ffi::Result::Success => Ok(()),
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::InvalidOpaqueCaptureAddress => Err(Error::InvalidOpaqueCaptureAddress),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        if self.user {
            unsafe { ffi::vkDestroyImage(self.device.handle, self.handle, ptr::null()) };
        }
    }
}

pub enum ImageViewType {
    OneDim,
    TwoDim,
    ThreeDim,
    Cube,
    OneDimArray,
    TwoDimArray,
    ThreeDimArray,
}

pub enum ComponentSwizzle {
    Identity,
    Zero,
    One,
    R,
    G,
    B,
    A,
}
pub struct ComponentMapping {
    pub r: ComponentSwizzle,
    pub g: ComponentSwizzle,
    pub b: ComponentSwizzle,
    pub a: ComponentSwizzle,
}

pub struct ImageSubresourceRange {
    pub aspect_mask: u32,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

pub struct ImageViewCreateInfo<'a> {
    pub image: &'a Image,
    pub view_type: ImageViewType,
    pub format: Format,
    pub components: ComponentMapping,
    pub subresource_range: ImageSubresourceRange,
}

pub struct ImageView {
    device: Rc<Device>,
    handle: ffi::ImageView,
}

impl ImageView {
    pub fn new(device: Rc<Device>, create_info: ImageViewCreateInfo) -> Result<Self, Error> {
        //TODO change these to From impl
        let view_type = match create_info.view_type {
            ImageViewType::OneDim => ffi::ImageViewType::OneDim,
            ImageViewType::TwoDim => ffi::ImageViewType::TwoDim,
            ImageViewType::ThreeDim => ffi::ImageViewType::ThreeDim,
            ImageViewType::Cube => ffi::ImageViewType::Cube,
            ImageViewType::OneDimArray => ffi::ImageViewType::OneDimArray,
            ImageViewType::TwoDimArray => ffi::ImageViewType::TwoDimArray,
            ImageViewType::ThreeDimArray => ffi::ImageViewType::ThreeDimArray,
        };

        let format = create_info.format.into();

        //TODO convert to From<non-ffi> for ffi
        let swizzle_f = |component| match component {
            ComponentSwizzle::Identity => ffi::ComponentSwizzle::Identity,
            ComponentSwizzle::Zero => ffi::ComponentSwizzle::Zero,
            ComponentSwizzle::One => ffi::ComponentSwizzle::One,
            ComponentSwizzle::R => ffi::ComponentSwizzle::R,
            ComponentSwizzle::G => ffi::ComponentSwizzle::G,
            ComponentSwizzle::B => ffi::ComponentSwizzle::B,
            ComponentSwizzle::A => ffi::ComponentSwizzle::A,
        };

        let components = ffi::ComponentMapping {
            r: swizzle_f(create_info.components.r),
            g: swizzle_f(create_info.components.g),
            b: swizzle_f(create_info.components.b),
            a: swizzle_f(create_info.components.a),
        };

        let subresource_range = ffi::ImageSubresourceRange {
            aspect_mask: create_info.subresource_range.aspect_mask,
            base_mip_level: create_info.subresource_range.base_mip_level,
            level_count: create_info.subresource_range.level_count,
            base_array_layer: create_info.subresource_range.base_array_layer,
            layer_count: create_info.subresource_range.layer_count,
        };

        let create_info = ffi::ImageViewCreateInfo {
            structure_type: ffi::StructureType::ImageViewCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            image: create_info.image.handle,
            view_type,
            format,
            components,
            subresource_range,
        };

        let mut handle = MaybeUninit::<ffi::ImageView>::uninit();

        let result = unsafe {
            ffi::vkCreateImageView(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let image_view = Self { device, handle };

                Ok(image_view)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyImageView(self.device.handle, self.handle, ptr::null()) };
    }
}

pub struct ShaderModuleCreateInfo<'a> {
    pub code: &'a [u32],
}

pub struct ShaderModule {
    device: Rc<Device>,
    handle: ffi::ShaderModule,
}

impl ShaderModule {
    pub fn new(device: Rc<Device>, create_info: ShaderModuleCreateInfo<'_>) -> Result<Self, Error> {
        let create_info = ffi::ShaderModuleCreateInfo {
            structure_type: ffi::StructureType::ShaderModuleCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            code_size: create_info.code.len() * mem::size_of::<u32>(),
            code: create_info.code.as_ptr(),
        };

        let mut handle = MaybeUninit::<ffi::ShaderModule>::uninit();

        let result = unsafe {
            ffi::vkCreateShaderModule(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let shader_module = Self { device, handle };

                Ok(shader_module)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::InvalidShader => Err(Error::InvalidShader),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyShaderModule(self.device.handle, self.handle, ptr::null()) };
    }
}

pub struct PipelineShaderStageCreateInfo<'a> {
    pub stage: u32,
    pub module: &'a ShaderModule,
    pub entry_point: &'a str,
}

#[derive(Clone, Copy, Debug)]
pub enum VertexInputRate {
    Vertex = 0,
    Instance = 1,
}

#[derive(Clone, Copy, Debug)]
pub struct VertexInputBindingDescription {
    pub binding: u32,
    pub stride: usize,
    pub input_rate: VertexInputRate,
}

#[derive(Clone, Copy, Debug)]
pub struct VertexInputAttributeDescription {
    pub location: u32,
    pub binding: u32,
    pub format: Format,
    pub offset: u32,
}

pub struct PipelineVertexInputStateCreateInfo<'a> {
    pub bindings: &'a [VertexInputBindingDescription],
    pub attributes: &'a [VertexInputAttributeDescription],
}

#[derive(Clone, Copy)]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

pub struct PipelineInputAssemblyStateCreateInfo {
    pub topology: PrimitiveTopology,
    pub primitive_restart_enable: bool,
}

pub struct PipelineTessellationStateCreateInfo {}

pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[derive(Copy, Clone)]
pub struct Rect2d {
    pub offset: Offset2d,
    pub extent: Extent2d,
}

pub struct PipelineViewportStateCreateInfo<'a> {
    pub viewports: &'a [Viewport],
    pub scissors: &'a [Rect2d],
}

#[derive(Clone, Copy)]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}

#[derive(Clone, Copy)]
pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

pub struct PipelineRasterizationStateCreateInfo {
    pub depth_clamp_enable: bool,
    pub rasterizer_discard_enable: bool,
    pub polygon_mode: PolygonMode,
    pub cull_mode: u32,
    pub front_face: FrontFace,
    pub depth_bias_enable: bool,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_clamp: f32,
    pub depth_bias_slope_factor: f32,
    pub line_width: f32,
}

pub struct PipelineMultisampleStateCreateInfo {}

pub struct PipelineDepthStencilStateCreateInfo {
    pub depth_test_enable: bool,
    pub depth_write_enable: bool,
    pub depth_compare_op: CompareOp,
    pub depth_bounds_test_enable: bool,
    pub min_depth_bounds: f32,
    pub max_depth_bounds: f32,
}

#[derive(Clone, Copy)]
pub enum BlendFactor {
    One,
    Zero,
    SrcAlpha,
    OneMinusSrcAlpha,
}

#[derive(Clone, Copy)]
pub enum BlendOp {
    Add,
}

pub struct PipelineColorBlendAttachmentState {
    pub color_write_mask: u32,
    pub blend_enable: bool,
    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_op: BlendOp,
    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub alpha_blend_op: BlendOp,
}

#[derive(Clone, Copy)]
pub enum LogicOp {
    Copy,
}

pub struct PipelineColorBlendStateCreateInfo<'a> {
    pub logic_op_enable: bool,
    pub logic_op: LogicOp,
    pub attachments: &'a [PipelineColorBlendAttachmentState],
    pub blend_constants: &'a [f32; 4],
}

#[derive(Clone, Copy)]
pub enum DynamicState {
    Viewport,
}

pub struct PipelineDynamicStateCreateInfo<'a> {
    pub dynamic_states: &'a [DynamicState],
}

pub struct PipelineLayoutCreateInfo<'a> {
    pub set_layouts: &'a [&'a DescriptorSetLayout],
}

pub struct PipelineLayout {
    device: Rc<Device>,
    handle: ffi::PipelineLayout,
}

impl PipelineLayout {
    pub fn new(device: Rc<Device>, create_info: PipelineLayoutCreateInfo) -> Result<Self, Error> {
        let set_layouts = create_info
            .set_layouts
            .iter()
            .map(|set_layout| set_layout.handle)
            .collect::<Vec<_>>();

        let create_info = ffi::PipelineLayoutCreateInfo {
            structure_type: ffi::StructureType::PipelineLayoutCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            set_layout_count: create_info.set_layouts.len() as _,
            set_layouts: set_layouts.as_ptr(),
            push_constant_range_count: 0,
            push_constant_ranges: ptr::null(),
        };

        let mut handle = MaybeUninit::<ffi::PipelineLayout>::uninit();

        let result = unsafe {
            ffi::vkCreatePipelineLayout(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let pipeline_layout = Self { device, handle };

                Ok(pipeline_layout)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyPipelineLayout(self.device.handle, self.handle, ptr::null()) };
    }
}

#[derive(Clone, Copy)]
pub enum AttachmentLoadOp {
    Load,
    Clear,
    DontCare,
}

#[derive(Clone, Copy)]
pub enum AttachmentStoreOp {
    Store,
    DontCare,
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

#[derive(Clone, Copy)]
pub struct AttachmentDescription {
    pub format: Format,
    pub samples: u32,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub stencil_load_op: AttachmentLoadOp,
    pub stencil_store_op: AttachmentStoreOp,
    pub initial_layout: ImageLayout,
    pub final_layout: ImageLayout,
}

#[derive(Clone, Copy)]
pub struct AttachmentReference {
    pub attachment: u32,
    pub layout: ImageLayout,
}

#[derive(Clone, Copy)]
pub enum PipelineBindPoint {
    Graphics,
    Compute,
}

#[derive(Clone, Copy)]
pub struct SubpassDescription<'a> {
    pub pipeline_bind_point: PipelineBindPoint,
    pub input_attachments: &'a [AttachmentReference],
    pub color_attachments: &'a [AttachmentReference],
    pub resolve_attachments: &'a [AttachmentReference],
    pub depth_stencil_attachment: Option<&'a AttachmentReference>,
    pub preserve_attachments: &'a [u32],
}

pub struct SubpassDependency {
    pub src_subpass: u32,
    pub dst_subpass: u32,
    pub src_stage_mask: u32,
    pub dst_stage_mask: u32,
    pub src_access_mask: u32,
    pub dst_access_mask: u32,
}

pub struct RenderPassCreateInfo<'a> {
    pub attachments: &'a [AttachmentDescription],
    pub subpasses: &'a [SubpassDescription<'a>],
    pub dependencies: &'a [SubpassDependency],
}

pub struct RenderPass {
    device: Rc<Device>,
    handle: ffi::RenderPass,
}

impl RenderPass {
    pub fn new(device: Rc<Device>, create_info: RenderPassCreateInfo<'_>) -> Result<Self, Error> {
        let attachment_descriptions = create_info
            .attachments
            .iter()
            .map(|&attachment| attachment.into())
            .collect::<Vec<_>>();

        let input_attachments = create_info
            .subpasses
            .iter()
            .map(|subpass| {
                subpass
                    .input_attachments
                    .iter()
                    .map(|&attachment| attachment.into())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let color_attachments = create_info
            .subpasses
            .iter()
            .map(|subpass| {
                subpass
                    .color_attachments
                    .iter()
                    .map(|&attachment| attachment.into())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let resolve_attachments = create_info
            .subpasses
            .iter()
            .map(|subpass| {
                subpass
                    .resolve_attachments
                    .iter()
                    .map(|&attachment| attachment.into())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let depth_stencil_attachments = create_info
            .subpasses
            .iter()
            .map(|subpass| {
                subpass
                    .depth_stencil_attachment
                    .map(|&attachment| attachment.into())
            })
            .collect::<Vec<_>>();

        let preserve_attachments = create_info
            .subpasses
            .iter()
            .map(|subpass| {
                subpass
                    .preserve_attachments
                    .iter()
                    .map(|&attachment| attachment as _)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let subpasses = create_info
            .subpasses
            .iter()
            .enumerate()
            .map(|(i, subpass)| {
                let input_attachment_count = input_attachments[i].len() as u32;

                let input_attachments = if input_attachment_count > 0 {
                    input_attachments[i].as_ptr()
                } else {
                    ptr::null()
                };

                let color_attachment_count = color_attachments[i].len() as u32;

                let color_attachments = if color_attachment_count > 0 {
                    color_attachments[i].as_ptr()
                } else {
                    ptr::null()
                };

                let resolve_attachment_count = resolve_attachments[i].len() as u32;

                let resolve_attachments = if resolve_attachment_count > 0 {
                    resolve_attachments[i].as_ptr()
                } else {
                    ptr::null()
                };

                let depth_stencil_attachment =
                    if let Some(attachment) = &depth_stencil_attachments[i] {
                        attachment
                    } else {
                        ptr::null()
                    };

                let preserve_attachment_count = preserve_attachments[i].len() as u32;

                let preserve_attachments = if preserve_attachment_count > 0 {
                    preserve_attachments[i].as_ptr()
                } else {
                    ptr::null()
                };

                ffi::SubpassDescription {
                    flags: 0,
                    pipeline_bind_point: subpass.pipeline_bind_point.into(),
                    input_attachment_count,
                    input_attachments,
                    color_attachment_count,
                    color_attachments,
                    resolve_attachments,
                    depth_stencil_attachment,
                    preserve_attachment_count,
                    preserve_attachments,
                }
            })
            .collect::<Vec<_>>();

        let dependencies = create_info
            .dependencies
            .iter()
            .map(|dependency| ffi::SubpassDependency {
                src_subpass: dependency.src_subpass,
                dst_subpass: dependency.dst_subpass,
                src_stage_mask: dependency.src_stage_mask,
                dst_stage_mask: dependency.dst_stage_mask,
                src_access_mask: dependency.src_access_mask,
                dst_access_mask: dependency.dst_access_mask,
                dependency_flags: 0,
            })
            .collect::<Vec<_>>();

        let create_info = ffi::RenderPassCreateInfo {
            structure_type: ffi::StructureType::RenderPassCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            attachment_count: attachment_descriptions.len() as _,
            attachments: attachment_descriptions.as_ptr(),
            subpass_count: subpasses.len() as _,
            subpasses: subpasses.as_ptr(),
            dependency_count: dependencies.len() as _,
            dependencies: dependencies.as_ptr(),
        };

        let mut handle = MaybeUninit::<ffi::RenderPass>::uninit();

        let result = unsafe {
            ffi::vkCreateRenderPass(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let render_pass = Self { device, handle };

                Ok(render_pass)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyRenderPass(self.device.handle, self.handle, ptr::null()) };
    }
}

pub struct ComputePipelineCreateInfo<'a> {
    pub stage: PipelineShaderStageCreateInfo<'a>,
    pub layout: &'a PipelineLayout,
    pub base_pipeline: Option<Pipeline>,
    pub base_pipeline_index: i32,
}

pub struct GraphicsPipelineCreateInfo<'a> {
    pub stages: &'a [PipelineShaderStageCreateInfo<'a>],
    pub vertex_input_state: &'a PipelineVertexInputStateCreateInfo<'a>,
    pub input_assembly_state: &'a PipelineInputAssemblyStateCreateInfo,
    pub tessellation_state: &'a PipelineTessellationStateCreateInfo,
    pub viewport_state: &'a PipelineViewportStateCreateInfo<'a>,
    pub rasterization_state: &'a PipelineRasterizationStateCreateInfo,
    pub multisample_state: &'a PipelineMultisampleStateCreateInfo,
    pub depth_stencil_state: &'a PipelineDepthStencilStateCreateInfo,
    pub color_blend_state: &'a PipelineColorBlendStateCreateInfo<'a>,
    pub dynamic_state: &'a PipelineDynamicStateCreateInfo<'a>,
    pub layout: &'a PipelineLayout,
    pub render_pass: &'a RenderPass,
    pub subpass: u32,
    pub base_pipeline: Option<Pipeline>,
    pub base_pipeline_index: i32,
}

pub struct PipelineCache {
    handle: ffi::PipelineCache,
}

pub struct Pipeline {
    device: Rc<Device>,
    handle: ffi::Pipeline,
}

impl Pipeline {
    pub fn new_graphics_pipelines(
        device: Rc<Device>,
        cache: Option<PipelineCache>,
        create_infos: &'_ [GraphicsPipelineCreateInfo],
    ) -> Result<Vec<Self>, Error> {
        let entry_points = create_infos
            .iter()
            .map(|create_info| {
                create_info
                    .stages
                    .iter()
                    .map(|stage| CString::new(stage.entry_point).unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let stages = create_infos
            .iter()
            .enumerate()
            .map(|(i, create_info)| {
                create_info
                    .stages
                    .iter()
                    .enumerate()
                    .map(|(j, stage)| ffi::PipelineShaderStageCreateInfo {
                        structure_type: ffi::StructureType::PipelineShaderStageCreateInfo,
                        p_next: ptr::null(),
                        flags: 0,
                        stage: stage.stage as _,
                        module: stage.module.handle,
                        entry_point: entry_points[i][j].as_ptr(),
                        specialization_info: ptr::null(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let vertex_binding_descriptions = create_infos
            .iter()
            .map(|create_info| {
                create_info
                    .vertex_input_state
                    .bindings
                    .iter()
                    .map(|binding| ffi::VertexInputBindingDescription {
                        binding: binding.binding,
                        stride: binding.stride as _,
                        input_rate: binding.input_rate.into(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let vertex_attribute_descriptions = create_infos
            .iter()
            .map(|create_info| {
                create_info
                    .vertex_input_state
                    .attributes
                    .iter()
                    .map(|attribute| ffi::VertexInputAttributeDescription {
                        binding: attribute.binding,
                        location: attribute.location,
                        format: attribute.format.into(),
                        offset: attribute.offset,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let vertex_input_states = create_infos
            .iter()
            .enumerate()
            .map(|(i, _)| ffi::PipelineVertexInputStateCreateInfo {
                structure_type: ffi::StructureType::PipelineVertexInputStateCreateInfo,
                p_next: ptr::null(),
                flags: 0,
                vertex_binding_description_count: vertex_binding_descriptions[i].len() as _,
                vertex_binding_descriptions: vertex_binding_descriptions[i].as_ptr(),
                vertex_attribute_description_count: vertex_attribute_descriptions[i].len() as _,
                vertex_attribute_descriptions: vertex_attribute_descriptions[i].as_ptr(),
            })
            .collect::<Vec<_>>();

        let input_assembly_states = create_infos
            .iter()
            .map(|create_info| ffi::PipelineInputAssemblyStateCreateInfo {
                structure_type: ffi::StructureType::PipelineInputAssemblyStateCreateInfo,
                p_next: ptr::null(),
                flags: 0,
                topology: create_info.input_assembly_state.topology.into(),
                primitive_restart_enable: create_info.input_assembly_state.primitive_restart_enable
                    as _,
            })
            .collect::<Vec<_>>();

        //TODO
        let tessellation_states = 0;

        let viewports = create_infos
            .iter()
            .map(|create_info| {
                create_info
                    .viewport_state
                    .viewports
                    .iter()
                    .map(|viewport| ffi::Viewport {
                        x: viewport.x,
                        y: viewport.y,
                        width: viewport.width,
                        height: viewport.height,
                        min_depth: viewport.min_depth,
                        max_depth: viewport.max_depth,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let scissors = create_infos
            .iter()
            .map(|create_info| {
                create_info
                    .viewport_state
                    .scissors
                    .iter()
                    .map(|scissor| ffi::Rect2d {
                        offset: [scissor.offset.0, scissor.offset.1],
                        extent: [scissor.extent.0, scissor.extent.1],
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let viewport_states = create_infos
            .iter()
            .enumerate()
            .map(|(i, create_info)| {
                let viewport_count = viewports[i].len() as _;

                let viewports = if viewport_count > 0 {
                    viewports[i].as_ptr()
                } else {
                    ptr::null()
                };

                let scissor_count = scissors[i].len() as _;

                let scissors = if scissor_count > 0 {
                    scissors[i].as_ptr()
                } else {
                    ptr::null()
                };

                ffi::PipelineViewportStateCreateInfo {
                    structure_type: ffi::StructureType::PipelineViewportStateCreateInfo,
                    p_next: ptr::null(),
                    flags: 0,
                    viewport_count,
                    viewports,
                    scissor_count,
                    scissors,
                }
            })
            .collect::<Vec<_>>();

        let rasterization_states = create_infos
            .iter()
            .map(|create_info| ffi::PipelineRasterizationStateCreateInfo {
                structure_type: ffi::StructureType::PipelineRasterizationStateCreateInfo,
                p_next: ptr::null(),
                flags: 0,
                depth_clamp_enable: create_info.rasterization_state.depth_clamp_enable as _,
                rasterizer_discard_enable: create_info.rasterization_state.rasterizer_discard_enable
                    as _,
                polygon_mode: create_info.rasterization_state.polygon_mode.into(),
                cull_mode: create_info.rasterization_state.cull_mode,
                front_face: create_info.rasterization_state.front_face.into(),
                depth_bias_enable: create_info.rasterization_state.depth_bias_enable as _,
                depth_bias_constant_factor: create_info
                    .rasterization_state
                    .depth_bias_constant_factor,
                depth_bias_clamp: create_info.rasterization_state.depth_bias_clamp,
                depth_bias_slope_factor: create_info.rasterization_state.depth_bias_slope_factor,
                line_width: create_info.rasterization_state.line_width,
            })
            .collect::<Vec<_>>();

        let multisample_states = create_infos
            .iter()
            .map(|create_info| ffi::PipelineMultisampleStateCreateInfo {
                structure_type: ffi::StructureType::PipelineMultisampleStateCreateInfo,
                p_next: ptr::null(),
                flags: 0,
                //Disable
                rasterization_samples: 0x00000001,
                sample_shading_enable: false as _,
                min_sample_shading: 1.0,
                sample_mask: ptr::null(),
                alpha_to_coverage_enable: false as _,
                alpha_to_one_enable: false as _,
            })
            .collect::<Vec<_>>();

        //TODO
        let depth_stencil_states = create_infos
            .iter()
            .map(|create_info| ffi::PipelineDepthStencilStateCreateInfo {
                structure_type: ffi::StructureType::PipelineDepthStencilStateCreateInfo,
                p_next: ptr::null(),
                flags: 0,
                depth_test_enable: create_info.depth_stencil_state.depth_test_enable as _,
                depth_write_enable: create_info.depth_stencil_state.depth_write_enable as _,
                depth_compare_op: create_info.depth_stencil_state.depth_compare_op.into(),
                depth_bounds_test_enable: create_info.depth_stencil_state.depth_bounds_test_enable
                    as _,
                //TODO stencil
                stencil_test_enable: false as _,
                front: ffi::StencilOpState {
                    fail_op: ffi::StencilOp::Keep,
                    pass_op: ffi::StencilOp::Keep,
                    depth_fail_op: ffi::StencilOp::Keep,
                    compare_op: ffi::CompareOp::Never,
                    compare_mask: 0,
                    write_mask: 0,
                    reference: 0,
                },
                back: ffi::StencilOpState {
                    fail_op: ffi::StencilOp::Keep,
                    pass_op: ffi::StencilOp::Keep,
                    depth_fail_op: ffi::StencilOp::Keep,
                    compare_op: ffi::CompareOp::Never,
                    compare_mask: 0,
                    write_mask: 0,
                    reference: 0,
                },
                min_depth_bounds: create_info.depth_stencil_state.min_depth_bounds as _,
                max_depth_bounds: create_info.depth_stencil_state.max_depth_bounds as _,
            })
            .collect::<Vec<_>>();

        let color_blend_attachment_states = create_infos
            .iter()
            .map(|create_info| {
                create_info
                    .color_blend_state
                    .attachments
                    .iter()
                    .map(|attachment| ffi::PipelineColorBlendAttachmentState {
                        blend_enable: attachment.blend_enable as _,
                        src_color_blend_factor: attachment.src_color_blend_factor.into(),
                        dst_color_blend_factor: attachment.dst_color_blend_factor.into(),
                        color_blend_op: attachment.color_blend_op.into(),
                        src_alpha_blend_factor: attachment.src_alpha_blend_factor.into(),
                        dst_alpha_blend_factor: attachment.dst_alpha_blend_factor.into(),
                        alpha_blend_op: attachment.alpha_blend_op.into(),
                        color_write_mask: attachment.color_write_mask,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let color_blend_states = create_infos
            .iter()
            .enumerate()
            .map(|(i, create_info)| {
                let attachment_count = create_info.color_blend_state.attachments.len() as _;

                let attachments = if attachment_count > 0 {
                    color_blend_attachment_states[i].as_ptr()
                } else {
                    ptr::null()
                };

                ffi::PipelineColorBlendStateCreateInfo {
                    structure_type: ffi::StructureType::PipelineColorBlendStateCreateInfo,
                    p_next: ptr::null(),
                    flags: 0,
                    logic_op_enable: create_info.color_blend_state.logic_op_enable as _,
                    logic_op: create_info.color_blend_state.logic_op.into(),
                    attachment_count,
                    attachments,
                    blend_constants: [
                        create_info.color_blend_state.blend_constants[0],
                        create_info.color_blend_state.blend_constants[1],
                        create_info.color_blend_state.blend_constants[2],
                        create_info.color_blend_state.blend_constants[3],
                    ],
                }
            })
            .collect::<Vec<_>>();

        let dynamic_state_data = create_infos
            .iter()
            .map(|create_info| {
                create_info
                    .dynamic_state
                    .dynamic_states
                    .iter()
                    .map(|&dynamic_state| dynamic_state.into())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let dynamic_states = create_infos
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let dynamic_state_count = dynamic_state_data[i].len() as _;

                let dynamic_states = if dynamic_state_count > 0 {
                    dynamic_state_data[i].as_ptr()
                } else {
                    ptr::null()
                };

                ffi::PipelineDynamicStateCreateInfo {
                    structure_type: ffi::StructureType::PipelineDynamicStateCreateInfo,
                    p_next: ptr::null(),
                    flags: 0,
                    dynamic_state_count,
                    dynamic_states,
                }
            })
            .collect::<Vec<_>>();

        let create_infos = create_infos
            .iter()
            .enumerate()
            .map(|(i, create_info)| ffi::GraphicsPipelineCreateInfo {
                structure_type: ffi::StructureType::GraphicsPipelineCreateInfo,
                p_next: ptr::null(),
                flags: 0,
                stage_count: stages[i].len() as _,
                stages: stages[i].as_ptr(),
                vertex_input_state: &vertex_input_states[i],
                input_assembly_state: &input_assembly_states[i],
                tessellation_state: ptr::null(),
                viewport_state: &viewport_states[i],
                rasterization_state: &rasterization_states[i],
                multisample_state: &multisample_states[i],
                depth_stencil_state: &depth_stencil_states[i],
                color_blend_state: &color_blend_states[i],
                dynamic_state: &dynamic_states[i],
                layout: create_info.layout.handle,
                render_pass: create_info.render_pass.handle,
                subpass: create_info.subpass as _,
                base_pipeline_handle: create_info
                    .base_pipeline
                    .as_ref()
                    .map_or(ffi::Pipeline::null(), |pipeline| pipeline.handle),
                base_pipeline_index: create_info.base_pipeline_index,
            })
            .collect::<Vec<_>>();

        let mut handles = Vec::with_capacity(create_infos.len());

        let result = unsafe {
            ffi::vkCreateGraphicsPipelines(
                device.handle,
                ffi::PipelineCache::null(),
                create_infos.len() as _,
                create_infos.as_ptr(),
                ptr::null(),
                handles.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                unsafe { handles.set_len(create_infos.len()) };

                let pipelines = handles
                    .into_iter()
                    .map(|handle| Pipeline {
                        device: device.clone(),
                        handle,
                    })
                    .collect::<Vec<_>>();

                Ok(pipelines)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::InvalidShader => Err(Error::InvalidShader),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn new_compute_pipelines(
        device: Rc<Device>,
        cache: Option<PipelineCache>,
        create_infos: &'_ [ComputePipelineCreateInfo],
    ) -> Result<Vec<Self>, Error> {
        let entry_points = create_infos
            .iter()
            .map(|create_info| CString::new(create_info.stage.entry_point).unwrap())
            .collect::<Vec<_>>();

        let stages = create_infos
            .iter()
            .enumerate()
            .map(|(i, create_info)| ffi::PipelineShaderStageCreateInfo {
                structure_type: ffi::StructureType::PipelineShaderStageCreateInfo,
                p_next: ptr::null(),
                flags: 0,
                stage: create_info.stage.stage as _,
                module: create_info.stage.module.handle,
                entry_point: entry_points[i].as_ptr(),
                specialization_info: ptr::null(),
            })
            .collect::<Vec<_>>();

        let create_infos = create_infos
            .iter()
            .enumerate()
            .map(|(i, create_info)| ffi::ComputePipelineCreateInfo {
                structure_type: ffi::StructureType::ComputePipelineCreateInfo,
                p_next: ptr::null(),
                flags: 0,
                stage: stages[i],
                layout: create_info.layout.handle,
                base_pipeline_handle: create_info
                    .base_pipeline
                    .as_ref()
                    .map_or(ffi::Pipeline::null(), |pipeline| pipeline.handle),
                base_pipeline_index: create_info.base_pipeline_index,
            })
            .collect::<Vec<_>>();

        let mut handles = Vec::with_capacity(create_infos.len());

        let result = unsafe {
            ffi::vkCreateComputePipelines(
                device.handle,
                ffi::PipelineCache::null(),
                create_infos.len() as _,
                create_infos.as_ptr(),
                ptr::null(),
                handles.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                unsafe { handles.set_len(create_infos.len()) };

                let pipelines = handles
                    .into_iter()
                    .map(|handle| Pipeline {
                        device: device.clone(),
                        handle,
                    })
                    .collect::<Vec<_>>();

                Ok(pipelines)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::InvalidShader => Err(Error::InvalidShader),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyPipeline(self.device.handle, self.handle, ptr::null()) };
    }
}

pub struct FramebufferCreateInfo<'a> {
    pub render_pass: &'a RenderPass,
    pub attachments: &'a [&'a ImageView],
    pub width: u32,
    pub height: u32,
    pub layers: u32,
}

pub struct Framebuffer {
    device: Rc<Device>,
    handle: ffi::Framebuffer,
}

impl Framebuffer {
    pub fn new(device: Rc<Device>, create_info: FramebufferCreateInfo) -> Result<Self, Error> {
        let attachments = create_info
            .attachments
            .iter()
            .map(|image_view| image_view.handle)
            .collect::<Vec<_>>();

        let create_info = ffi::FramebufferCreateInfo {
            structure_type: ffi::StructureType::FramebufferCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            render_pass: create_info.render_pass.handle,
            attachment_count: create_info.attachments.len() as _,
            attachments: attachments.as_ptr(),
            width: create_info.width,
            height: create_info.height,
            layers: create_info.layers,
        };

        let mut handle = MaybeUninit::<ffi::Framebuffer>::uninit();

        let result = unsafe {
            ffi::vkCreateFramebuffer(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let framebuffer = Self { device, handle };

                Ok(framebuffer)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyFramebuffer(self.device.handle, self.handle, ptr::null()) };
    }
}

pub struct CommandPoolCreateInfo {
    pub queue_family_index: u32,
}

pub struct CommandPool {
    device: Rc<Device>,
    handle: ffi::CommandPool,
}

impl CommandPool {
    pub fn new(device: Rc<Device>, create_info: CommandPoolCreateInfo) -> Result<Self, Error> {
        let create_info = ffi::CommandPoolCreateInfo {
            structure_type: ffi::StructureType::CommandPoolCreateInfo,
            p_next: ptr::null(),
            flags: 0x00000002,
            queue_family_index: create_info.queue_family_index,
        };

        let mut handle = MaybeUninit::<ffi::CommandPool>::uninit();

        let result = unsafe {
            ffi::vkCreateCommandPool(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let command_pool = Self { device, handle };

                Ok(command_pool)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyCommandPool(self.device.handle, self.handle, ptr::null()) };
    }
}

#[derive(Clone, Copy)]
pub enum CommandBufferLevel {
    Primary,
    Secondary,
}

pub struct CommandBufferAllocateInfo<'a> {
    pub command_pool: &'a CommandPool,
    pub level: CommandBufferLevel,
    pub count: u32,
}

pub struct CommandBuffer {
    device: Rc<Device>,
    handle: ffi::CommandBuffer,
}

impl CommandBuffer {
    pub fn allocate(
        device: Rc<Device>,
        allocate_info: CommandBufferAllocateInfo<'_>,
    ) -> Result<Vec<Self>, Error> {
        let allocate_info = ffi::CommandBufferAllocateInfo {
            structure_type: ffi::StructureType::CommandBufferAllocateInfo,
            p_next: ptr::null(),
            command_pool: allocate_info.command_pool.handle,
            level: allocate_info.level.into(),
            command_buffer_count: allocate_info.count,
        };

        let mut handles = Vec::with_capacity(allocate_info.command_buffer_count as _);

        let result = unsafe {
            ffi::vkAllocateCommandBuffers(device.handle, &allocate_info, handles.as_mut_ptr())
        };

        match result {
            ffi::Result::Success => {
                unsafe { handles.set_len(allocate_info.command_buffer_count as _) };

                let command_pools = handles
                    .into_iter()
                    .map(|handle| Self {
                        device: device.clone(),
                        handle,
                    })
                    .collect::<Vec<_>>();

                Ok(command_pools)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn record<'a>(&'a mut self, mut script: impl FnMut(Commands<'_>)) -> Result<(), Error> {
        let begin_info = ffi::CommandBufferBeginInfo {
            structure_type: ffi::StructureType::CommandBufferBeginInfo,
            p_next: ptr::null(),
            flags: 0,
            inheritence_info: ptr::null(),
        };

        let result = unsafe { ffi::vkBeginCommandBuffer(self.handle, &begin_info) };

        match result {
            ffi::Result::Success => {}
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory)?,
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory)?,
            _ => panic!("unexpected result: {:?}", result),
        }

        let commands = Commands {
            command_buffer: self,
        };

        script(commands);

        let result = unsafe { ffi::vkEndCommandBuffer(self.handle) };

        match result {
            ffi::Result::Success => Ok(()),
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        let result = unsafe { ffi::vkResetCommandBuffer(self.handle, 0) };

        match result {
            ffi::Result::Success => Ok(()),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

pub struct Commands<'a> {
    command_buffer: &'a mut CommandBuffer,
}

impl Commands<'_> {
    pub fn begin_render_pass(&mut self, begin_info: RenderPassBeginInfo<'_>) {
        let mut clear_values = vec![];

        clear_values.extend(
            begin_info
                .color_clear_values
                .into_iter()
                .map(|&clear_value| ffi::ClearValue {
                    color: ffi::ClearColorValue { rgba: clear_value },
                }),
        );

        if let Some(clear_value) = begin_info.depth_stencil_clear_value {
            let depth_stencil_clear_value = ffi::ClearValue {
                depth_stencil: ffi::ClearDepthStencilValue {
                    depth: clear_value.0,
                    stencil: clear_value.1,
                },
            };

            clear_values.push(depth_stencil_clear_value);
        }

        let begin_info = ffi::RenderPassBeginInfo {
            structure_type: ffi::StructureType::RenderPassBeginInfo,
            p_next: ptr::null(),
            render_pass: begin_info.render_pass.handle,
            framebuffer: begin_info.framebuffer.handle,
            render_area: ffi::Rect2d {
                offset: [
                    begin_info.render_area.offset.0,
                    begin_info.render_area.offset.1,
                ],
                extent: [
                    begin_info.render_area.extent.0,
                    begin_info.render_area.extent.1,
                ],
            },
            clear_value_count: clear_values.len() as _,
            clear_values: clear_values.as_ptr() as _,
        };

        unsafe {
            ffi::vkCmdBeginRenderPass(
                self.command_buffer.handle,
                &begin_info,
                ffi::SubpassContents::Inline,
            )
        };
    }

    pub fn end_render_pass(&mut self) {
        unsafe { ffi::vkCmdEndRenderPass(self.command_buffer.handle) };
    }

    pub fn next_subpass(&mut self) {
        unsafe { ffi::vkCmdNextSubpass(self.command_buffer.handle, ffi::SubpassContents::Inline) };
    }

    pub fn bind_pipeline(&mut self, bind_point: PipelineBindPoint, pipeline: &Pipeline) {
        unsafe {
            ffi::vkCmdBindPipeline(
                self.command_buffer.handle,
                bind_point.into(),
                pipeline.handle,
            )
        };
    }

    pub fn bind_descriptor_sets(
        &mut self,
        bind_point: PipelineBindPoint,
        layout: &'_ PipelineLayout,
        first_set: u32,
        descriptor_sets: &'_ [&'_ DescriptorSet],
        dynamic_offsets: &'_ [u32],
    ) {
        let descriptor_sets = descriptor_sets
            .iter()
            .map(|set| set.handle)
            .collect::<Vec<_>>();

        unsafe {
            ffi::vkCmdBindDescriptorSets(
                self.command_buffer.handle,
                bind_point.into(),
                layout.handle,
                first_set as _,
                descriptor_sets.len() as _,
                descriptor_sets.as_ptr(),
                dynamic_offsets.len() as _,
                dynamic_offsets.as_ptr() as _,
            )
        };
    }

    pub fn bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers: &'_ [&'_ Buffer],
        offsets: &'_ [usize],
    ) {
        let buffers = buffers
            .iter()
            .map(|buffer| buffer.handle)
            .collect::<Vec<_>>();
        let offsets = offsets
            .iter()
            .map(|&offset| offset as _)
            .collect::<Vec<_>>();

        assert_eq!(buffers.len(), offsets.len());

        unsafe {
            ffi::vkCmdBindVertexBuffers(
                self.command_buffer.handle,
                first_binding,
                buffers.len() as _,
                buffers.as_ptr(),
                offsets.as_ptr(),
            )
        };
    }

    pub fn bind_index_buffer(&mut self, buffer: &'_ Buffer, offset: usize, index_type: IndexType) {
        unsafe {
            ffi::vkCmdBindIndexBuffer(
                self.command_buffer.handle,
                buffer.handle,
                offset as _,
                index_type.into(),
            )
        };
    }

    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            ffi::vkCmdDraw(
                self.command_buffer.handle,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        };
    }

    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            ffi::vkCmdDrawIndexed(
                self.command_buffer.handle,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            )
        };
    }

    pub fn dispatch(&mut self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        unsafe {
            ffi::vkCmdDispatch(
                self.command_buffer.handle,
                group_count_x,
                group_count_y,
                group_count_z,
            )
        }
    }

    pub fn copy_buffer(
        &mut self,
        src_buffer: &Buffer,
        dst_buffer: &mut Buffer,
        regions: &'_ [BufferCopy],
    ) {
        let regions = regions
            .iter()
            .map(|copy| ffi::BufferCopy {
                src_offset: copy.src_offset,
                dst_offset: copy.dst_offset,
                size: copy.size,
            })
            .collect::<Vec<_>>();

        unsafe {
            ffi::vkCmdCopyBuffer(
                self.command_buffer.handle,
                src_buffer.handle,
                dst_buffer.handle,
                regions.len() as _,
                regions.as_ptr(),
            )
        };
    }

    pub fn copy_buffer_to_image(
        &mut self,
        src_buffer: &Buffer,
        dst_image: &mut Image,
        dst_image_layout: ImageLayout,
        regions: &'_ [BufferImageCopy],
    ) {
        let regions = regions
            .iter()
            .map(|copy| ffi::BufferImageCopy {
                buffer_offset: copy.buffer_offset as _,
                buffer_row_length: copy.buffer_row_length as _,
                buffer_image_height: copy.buffer_image_height as _,
                image_subresource: ffi::ImageSubresourceLayers {
                    aspect_mask: copy.image_subresource.aspect_mask as _,
                    mip_level: copy.image_subresource.mip_level as _,
                    base_array_layer: copy.image_subresource.base_array_layer as _,
                    layer_count: copy.image_subresource.layer_count as _,
                },
                image_offset: [
                    copy.image_offset.0 as _,
                    copy.image_offset.1 as _,
                    copy.image_offset.2 as _,
                ],
                image_extent: [
                    copy.image_extent.0 as _,
                    copy.image_extent.1 as _,
                    copy.image_extent.2 as _,
                ],
            })
            .collect::<Vec<_>>();

        unsafe {
            ffi::vkCmdCopyBufferToImage(
                self.command_buffer.handle,
                src_buffer.handle,
                dst_image.handle,
                dst_image_layout.into(),
                regions.len() as _,
                regions.as_ptr(),
            )
        };
    }

    pub fn pipeline_barrier(
        &mut self,
        src_stage_mask: u32,
        dst_stage_mask: u32,
        dependency_flags: u32,
        memory_barriers: &'_ [MemoryBarrier],
        buffer_memory_barriers: &'_ [BufferMemoryBarrier],
        image_memory_barriers: &'_ [ImageMemoryBarrier],
    ) {
        let memory_barriers = memory_barriers
            .iter()
            .map(|barrier| ffi::MemoryBarrier {
                structure_type: ffi::StructureType::MemoryBarrier,
                p_next: ptr::null(),
                src_access_mask: barrier.src_access_mask as _,
                dst_access_mask: barrier.dst_access_mask as _,
            })
            .collect::<Vec<_>>();

        let memory_barriers_ptr = if memory_barriers.len() > 0 {
            memory_barriers.as_ptr()
        } else {
            ptr::null()
        };

        let buffer_memory_barriers = buffer_memory_barriers
            .iter()
            .map(|barrier| ffi::BufferMemoryBarrier {
                structure_type: ffi::StructureType::BufferMemoryBarrier,
                p_next: ptr::null(),
                src_access_mask: barrier.src_access_mask as _,
                dst_access_mask: barrier.dst_access_mask as _,
                src_queue_family_index: barrier.src_queue_family_index as _,
                dst_queue_family_index: barrier.dst_queue_family_index as _,
                buffer: barrier.buffer.handle,
                offset: barrier.offset as _,
                size: barrier.size as _,
            })
            .collect::<Vec<_>>();

        let buffer_memory_barriers_ptr = if buffer_memory_barriers.len() > 0 {
            buffer_memory_barriers.as_ptr()
        } else {
            ptr::null()
        };

        let image_memory_barriers = image_memory_barriers
            .iter()
            .map(|barrier| ffi::ImageMemoryBarrier {
                structure_type: ffi::StructureType::ImageMemoryBarrier,
                p_next: ptr::null(),
                src_access_mask: barrier.src_access_mask as _,
                dst_access_mask: barrier.dst_access_mask as _,
                old_layout: barrier.old_layout.into(),
                new_layout: barrier.new_layout.into(),
                src_queue_family_index: barrier.src_queue_family_index as _,
                dst_queue_family_index: barrier.dst_queue_family_index as _,
                image: barrier.image.handle,
                subresource_range: ffi::ImageSubresourceRange {
                    aspect_mask: barrier.subresource_range.aspect_mask,
                    base_mip_level: barrier.subresource_range.base_mip_level,
                    level_count: barrier.subresource_range.level_count,
                    base_array_layer: barrier.subresource_range.base_array_layer,
                    layer_count: barrier.subresource_range.layer_count,
                },
            })
            .collect::<Vec<_>>();

        let image_memory_barriers_ptr = if image_memory_barriers.len() > 0 {
            image_memory_barriers.as_ptr()
        } else {
            ptr::null()
        };

        unsafe {
            ffi::vkCmdPipelineBarrier(
                self.command_buffer.handle,
                src_stage_mask as _,
                dst_stage_mask as _,
                dependency_flags as _,
                memory_barriers.len() as _,
                memory_barriers_ptr,
                buffer_memory_barriers.len() as _,
                buffer_memory_barriers_ptr,
                image_memory_barriers.len() as _,
                image_memory_barriers_ptr,
            )
        };
    }
}

pub struct RenderPassBeginInfo<'a> {
    pub render_pass: &'a RenderPass,
    pub framebuffer: &'a Framebuffer,
    pub render_area: Rect2d,
    pub color_clear_values: &'a [[f32; 4]],
    pub depth_stencil_clear_value: Option<(f32, u32)>,
}

pub struct SemaphoreCreateInfo {}

pub struct Semaphore {
    device: Rc<Device>,
    handle: ffi::Semaphore,
}

impl Semaphore {
    pub fn new(device: Rc<Device>, create_info: SemaphoreCreateInfo) -> Result<Self, Error> {
        let create_info = ffi::SemaphoreCreateInfo {
            structure_type: ffi::StructureType::SemaphoreCreateInfo,
            p_next: ptr::null(),
            flags: 0,
        };

        let mut handle = MaybeUninit::<ffi::Semaphore>::uninit();

        let result = unsafe {
            ffi::vkCreateSemaphore(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let semaphore = Self { device, handle };

                Ok(semaphore)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroySemaphore(self.device.handle, self.handle, ptr::null()) };
    }
}

pub struct FenceCreateInfo {}

pub struct Fence {
    device: Rc<Device>,
    handle: ffi::Fence,
}

impl Fence {
    pub fn new(device: Rc<Device>, create_info: FenceCreateInfo) -> Result<Self, Error> {
        let create_info = ffi::FenceCreateInfo {
            structure_type: ffi::StructureType::FenceCreateInfo,
            p_next: ptr::null(),
            flags: 0x00000001,
        };

        let mut handle = MaybeUninit::<ffi::Fence>::uninit();

        let result = unsafe {
            ffi::vkCreateFence(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let fence = Self { device, handle };

                Ok(fence)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn wait(fences: &'_ [&'_ mut Self], wait_all: bool, timeout: u64) -> Result<(), Error> {
        if fences.len() == 0 {
            return Ok(());
        }

        let same_device = fences
            .iter()
            .all(|fence| fence.device.handle == fences[0].device.handle);

        if !same_device {
            panic!("fences must be for same device");
        }

        let device_handle = fences[0].device.handle;

        let fences = fences.iter().map(|fence| fence.handle).collect::<Vec<_>>();

        let result = unsafe {
            ffi::vkWaitForFences(
                device_handle,
                fences.len() as _,
                fences.as_ptr(),
                wait_all as _,
                timeout as _,
            )
        };

        match result {
            ffi::Result::Success | ffi::Result::Timeout => Ok(()),
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::DeviceLost => Err(Error::DeviceLost),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn reset(fences: &'_ [&'_ mut Self]) -> Result<(), Error> {
        if fences.len() == 0 {
            return Ok(());
        }

        let same_device = fences
            .iter()
            .all(|fence| fence.device.handle == fences[0].device.handle);

        if !same_device {
            panic!("fences must be for same device");
        }

        let device_handle = fences[0].device.handle;

        let fences = fences.iter().map(|fence| fence.handle).collect::<Vec<_>>();

        let result =
            unsafe { ffi::vkResetFences(device_handle, fences.len() as _, fences.as_ptr()) };

        match result {
            ffi::Result::Success => Ok(()),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyFence(self.device.handle, self.handle, ptr::null()) };
    }
}

pub struct SubmitInfo<'a> {
    pub wait_semaphores: &'a [&'a Semaphore],
    pub wait_stages: &'a [u32],
    pub signal_semaphores: &'a [&'a mut Semaphore],
    pub command_buffers: &'a [&'a CommandBuffer],
}

pub struct PresentInfo<'a> {
    pub wait_semaphores: &'a [&'a Semaphore],
    pub swapchains: &'a [&'a Swapchain],
    pub image_indices: &'a [u32],
}

pub struct Buffer {
    device: Rc<Device>,
    handle: ffi::Buffer,
}

impl Buffer {
    pub fn new(device: Rc<Device>, size: u64, usage: u32) -> Result<Self, Error> {
        let create_info = ffi::BufferCreateInfo {
            structure_type: ffi::StructureType::BufferCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            size: size as _,
            usage: usage as _,
            sharing_mode: ffi::SharingMode::Exclusive,
            queue_family_index_count: 0,
            queue_family_indices: ptr::null(),
        };

        let mut handle = MaybeUninit::<ffi::Buffer>::uninit();

        let result = unsafe {
            ffi::vkCreateBuffer(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let buffer = Self { device, handle };

                Ok(buffer)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::InvalidOpaqueCaptureAddress => Err(Error::InvalidOpaqueCaptureAddress),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn memory_requirements(&self) -> MemoryRequirements {
        let mut memory_requirements = MaybeUninit::<ffi::MemoryRequirements>::uninit();

        unsafe {
            ffi::vkGetBufferMemoryRequirements(
                self.device.handle,
                self.handle,
                memory_requirements.as_mut_ptr(),
            )
        };

        let memory_requirements = unsafe { memory_requirements.assume_init() };

        MemoryRequirements {
            size: memory_requirements.size,
            alignment: memory_requirements.alignment,
            memory_type: memory_requirements.memory_type,
        }
    }

    pub fn bind_memory(&mut self, memory: &Memory) -> Result<(), Error> {
        let result =
            unsafe { ffi::vkBindBufferMemory(self.device.handle, self.handle, memory.handle, 0) };

        match result {
            ffi::Result::Success => Ok(()),
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::InvalidOpaqueCaptureAddress => Err(Error::InvalidOpaqueCaptureAddress),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyBuffer(self.device.handle, self.handle, ptr::null()) };
    }
}

#[derive(Clone, Copy)]
pub enum DescriptorType {
    CombinedImageSampler,
    StorageImage,
    UniformBuffer,
    StorageBuffer,
}

pub struct DescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage: u32,
}

pub struct DescriptorSetLayoutCreateInfo<'a> {
    pub bindings: &'a [DescriptorSetLayoutBinding],
}

pub struct DescriptorSetLayout {
    device: Rc<Device>,
    handle: ffi::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn new(
        device: Rc<Device>,
        create_info: DescriptorSetLayoutCreateInfo<'_>,
    ) -> Result<Self, Error> {
        let bindings = create_info
            .bindings
            .iter()
            .map(|binding| ffi::DescriptorSetLayoutBinding {
                binding: binding.binding as _,
                descriptor_type: binding.descriptor_type.into(),
                descriptor_count: binding.descriptor_count as _,
                stage: binding.stage as _,
                immutable_samplers: ptr::null(),
            })
            .collect::<Vec<_>>();

        let create_info = ffi::DescriptorSetLayoutCreateInfo {
            structure_type: ffi::StructureType::DescriptorSetLayoutCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            binding_count: create_info.bindings.len() as _,
            bindings: bindings.as_ptr(),
        };

        let mut handle = MaybeUninit::<ffi::DescriptorSetLayout>::uninit();

        let result = unsafe {
            ffi::vkCreateDescriptorSetLayout(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let descriptor_set_layout = Self { device, handle };

                Ok(descriptor_set_layout)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyDescriptorSetLayout(self.device.handle, self.handle, ptr::null()) };
    }
}

pub struct DescriptorSetAllocateInfo<'a> {
    pub descriptor_pool: &'a DescriptorPool,
    pub set_layouts: &'a [&'a DescriptorSetLayout],
}

pub struct DescriptorSet {
    device: Rc<Device>,
    handle: ffi::DescriptorSet,
}

impl DescriptorSet {
    pub fn allocate(
        device: Rc<Device>,
        allocate_info: DescriptorSetAllocateInfo<'_>,
    ) -> Result<Vec<Self>, Error> {
        let set_layouts = allocate_info
            .set_layouts
            .iter()
            .map(|set_layout| set_layout.handle)
            .collect::<Vec<_>>();

        let allocate_info = ffi::DescriptorSetAllocateInfo {
            structure_type: ffi::StructureType::DescriptorSetAllocateInfo,
            p_next: ptr::null(),
            descriptor_pool: allocate_info.descriptor_pool.handle,
            descriptor_set_count: set_layouts.len() as _,
            set_layouts: set_layouts.as_ptr(),
        };

        let mut handles =
            Vec::<ffi::DescriptorSet>::with_capacity(allocate_info.descriptor_set_count as _);

        let result = unsafe {
            ffi::vkAllocateDescriptorSets(device.handle, &allocate_info, handles.as_mut_ptr())
        };

        match result {
            ffi::Result::Success => {
                unsafe { handles.set_len(allocate_info.descriptor_set_count as _) };

                let descriptor_sets = handles
                    .into_iter()
                    .map(|handle| Self {
                        device: device.clone(),
                        handle,
                    })
                    .collect::<Vec<_>>();

                Ok(descriptor_sets)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::FragmentedPool => Err(Error::FragmentedPool),
            ffi::Result::OutOfPoolMemory => Err(Error::OutOfPoolMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn update(writes: &'_ [WriteDescriptorSet], copies: &'_ [CopyDescriptorSet]) {
        if writes.len() == 0 && copies.len() == 0 {
            return;
        }

        let same_device_writes = writes
            .iter()
            .all(|write| write.dst_set.device.handle == writes[0].dst_set.device.handle);

        let same_device_copies = copies
            .iter()
            .all(|copy| copy.dst_set.device.handle == copies[0].dst_set.device.handle);

        if !same_device_writes || !same_device_copies {
            panic!("descriptor set write or copy must be for same device");
        }

        let device = if writes.len() > 0 {
            writes[0].dst_set.device.clone()
        } else {
            copies[0].dst_set.device.clone()
        };

        let write_buffer_infos = writes
            .iter()
            .map(|write| {
                write
                    .buffer_infos
                    .iter()
                    .map(|buffer_info| ffi::DescriptorBufferInfo {
                        buffer: buffer_info.buffer.handle,
                        offset: buffer_info.offset as _,
                        range: buffer_info.range as _,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let write_buffer_info_ptrs = writes
            .iter()
            .enumerate()
            .map(|(i, write)| {
                if write.buffer_infos.len() > 0 {
                    write_buffer_infos[i].as_ptr()
                } else {
                    ptr::null()
                }
            })
            .collect::<Vec<_>>();

        let write_image_infos = writes
            .iter()
            .map(|write| {
                write
                    .image_infos
                    .iter()
                    .map(|image_info| ffi::DescriptorImageInfo {
                        sampler: image_info.sampler.handle,
                        image_view: image_info.image_view.handle,
                        image_layout: image_info.image_layout.into(),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let write_image_info_ptrs = writes
            .iter()
            .enumerate()
            .map(|(i, write)| {
                if write.image_infos.len() > 0 {
                    write_image_infos[i].as_ptr()
                } else {
                    ptr::null()
                }
            })
            .collect::<Vec<_>>();

        let writes = writes
            .iter()
            .enumerate()
            .map(|(i, write)| ffi::WriteDescriptorSet {
                structure_type: ffi::StructureType::WriteDescriptorSet,
                p_next: ptr::null(),
                dst_set: write.dst_set.handle,
                dst_binding: write.dst_binding,
                dst_array_element: write.dst_array_element,
                descriptor_count: write.descriptor_count,
                descriptor_type: write.descriptor_type.into(),
                image_infos: write_image_info_ptrs[i],
                buffer_infos: write_buffer_info_ptrs[i],
                texel_buffer_view: ptr::null(),
            })
            .collect::<Vec<_>>();

        let copies = copies
            .iter()
            .map(|copy| ffi::CopyDescriptorSet {
                structure_type: ffi::StructureType::CopyDescriptorSet,
                p_next: ptr::null(),
                src_set: copy.src_set.handle,
                src_binding: copy.src_binding,
                src_array_element: copy.src_array_element,
                dst_set: copy.dst_set.handle,
                dst_binding: copy.dst_binding,
                dst_array_element: copy.dst_array_element,
                descriptor_count: copy.descriptor_count,
            })
            .collect::<Vec<_>>();

        unsafe {
            ffi::vkUpdateDescriptorSets(
                device.handle,
                writes.len() as _,
                writes.as_ptr(),
                copies.len() as _,
                copies.as_ptr(),
            )
        };
    }
}

pub struct DescriptorBufferInfo<'a> {
    pub buffer: &'a Buffer,
    pub offset: usize,
    pub range: usize,
}

pub struct DescriptorImageInfo<'a> {
    pub sampler: &'a Sampler,
    pub image_view: &'a ImageView,
    pub image_layout: ImageLayout,
}

pub struct WriteDescriptorSet<'a> {
    pub dst_set: &'a DescriptorSet,
    pub dst_binding: u32,
    pub dst_array_element: u32,
    pub descriptor_count: u32,
    pub descriptor_type: DescriptorType,
    pub buffer_infos: &'a [DescriptorBufferInfo<'a>],
    pub image_infos: &'a [DescriptorImageInfo<'a>],
}

pub struct CopyDescriptorSet<'a> {
    pub src_set: &'a DescriptorSet,
    pub src_binding: u32,
    pub src_array_element: u32,
    pub dst_set: &'a DescriptorSet,
    pub dst_binding: u32,
    pub dst_array_element: u32,
    pub descriptor_count: u32,
}

pub struct DescriptorPoolSize {
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
}

pub struct DescriptorPoolCreateInfo<'a> {
    pub max_sets: u32,
    pub pool_sizes: &'a [DescriptorPoolSize],
}

pub struct DescriptorPool {
    device: Rc<Device>,
    handle: ffi::DescriptorPool,
}

impl DescriptorPool {
    pub fn new(
        device: Rc<Device>,
        create_info: DescriptorPoolCreateInfo<'_>,
    ) -> Result<Self, Error> {
        let pool_sizes = create_info
            .pool_sizes
            .iter()
            .map(|pool_size| ffi::DescriptorPoolSize {
                descriptor_type: pool_size.descriptor_type.into(),
                descriptor_count: pool_size.descriptor_count as _,
            })
            .collect::<Vec<_>>();

        let create_info = ffi::DescriptorPoolCreateInfo {
            structure_type: ffi::StructureType::DescriptorPoolCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            max_sets: create_info.max_sets,
            pool_size_count: create_info.pool_sizes.len() as _,
            pool_sizes: pool_sizes.as_ptr(),
        };

        let mut handle = MaybeUninit::<ffi::DescriptorPool>::uninit();

        let result = unsafe {
            ffi::vkCreateDescriptorPool(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let descriptor_pool = Self { device, handle };

                Ok(descriptor_pool)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::Fragmentation => Err(Error::Fragmentation),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyDescriptorPool(self.device.handle, self.handle, ptr::null()) };
    }
}

pub struct MemoryType {
    property_flags: u32,
    heap_index: u32,
}

pub struct MemoryHeap {
    size: u64,
    flags: u32,
}

pub struct MemoryProperties {
    memory_types: Vec<MemoryType>,
    memory_heaps: Vec<MemoryHeap>,
}

pub struct MemoryRequirements {
    size: u64,
    alignment: u64,
    memory_type: u32,
}

pub struct MemoryAllocateInfo {
    pub property_flags: u32,
}

pub struct Memory {
    device: Rc<Device>,
    handle: ffi::DeviceMemory,
    size: u64,
    mem: Option<*mut u8>,
}

impl Memory {
    pub fn allocate(
        device: Rc<Device>,
        allocate_info: MemoryAllocateInfo,
        requirements: MemoryRequirements,
        properties: MemoryProperties,
        mapped: bool,
    ) -> Result<Self, Error> {
        let size = requirements.size;

        let mut memory_type_index = None;

        for i in 0..properties.memory_types.len() {
            if requirements.memory_type & (1 << i) != 0
                && properties.memory_types[i as usize].property_flags & allocate_info.property_flags
                    != 0
            {
                memory_type_index = Some(i);
                break;
            }
        }

        let memory_type_index =
            memory_type_index.unwrap_or_else(|| panic!("couldnt find valid memory type")) as _;

        let allocate_info = ffi::MemoryAllocateInfo {
            structure_type: ffi::StructureType::MemoryAllocateInfo,
            p_next: ptr::null(),
            size,
            memory_type_index,
        };

        let mut handle = MaybeUninit::<ffi::DeviceMemory>::uninit();

        let result = unsafe {
            ffi::vkAllocateMemory(
                device.handle,
                &allocate_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let mem = if mapped {
                    let mut mem = ptr::null_mut::<u8>();

                    let result = unsafe {
                        ffi::vkMapMemory(device.handle, handle, 0, size, 0, &mut mem as *mut _ as _)
                    };

                    match result {
                        ffi::Result::Success => {}
                        ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory)?,
                        ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory)?,
                        ffi::Result::MemoryMapFailed => Err(Error::MemoryMapFailed)?,
                        _ => panic!("unexpected result: {:?}", result),
                    }

                    Some(mem)
                } else {
                    None
                };

                let memory = Self {
                    device,
                    handle,
                    size,
                    mem,
                };

                Ok(memory)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            ffi::Result::InvalidExternalHandle => Err(Error::InvalidExternalHandle),
            ffi::Result::InvalidOpaqueCaptureAddress => Err(Error::InvalidOpaqueCaptureAddress),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    pub fn write<'a, T: 'a>(
        &self,
        offset: usize,
        mut script: impl FnMut(&'a mut [T]),
    ) -> Result<(), Error> {
        if offset > self.size as _ {
            panic!("attempt to overflow buffer");
        }

        let mem = self.mem.ok_or(Error::MemoryMapFailed)?;

        let data = unsafe {
            slice::from_raw_parts_mut(
                mem.add(offset) as _,
                (self.size as usize - offset) / mem::size_of::<T>(),
            )
        };

        script(data);

        Ok(())
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        if let Some(_) = self.mem {
            unsafe { ffi::vkUnmapMemory(self.device.handle, self.handle) };
        }
        unsafe { ffi::vkFreeMemory(self.device.handle, self.handle, ptr::null()) };
    }
}

pub struct BufferCopy {
    pub src_offset: u64,
    pub dst_offset: u64,
    pub size: u64,
}

pub struct ImageSubresourceLayers {
    pub aspect_mask: u32,
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

pub struct BufferImageCopy {
    pub buffer_offset: u64,
    pub buffer_row_length: u32,
    pub buffer_image_height: u32,
    pub image_subresource: ImageSubresourceLayers,
    pub image_offset: Offset3d,
    pub image_extent: Extent3d,
}

#[derive(Clone, Copy)]
pub enum Filter {
    Nearest,
    Linear,
}

#[derive(Clone, Copy)]
pub enum SamplerMipmapMode {
    Nearest,
    Linear,
}

#[derive(Clone, Copy)]
pub enum SamplerAddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

#[derive(Clone, Copy)]
pub enum BorderColor {
    FloatTransparentBlack,
    IntTransparentBlack,
    FloatOpaqueBlack,
    IntOpaqueBlack,
    FloatOpaqueWhite,
    IntOpaqueWhite,
}

pub struct SamplerCreateInfo {
    pub mag_filter: Filter,
    pub min_filter: Filter,
    pub mipmap_mode: SamplerMipmapMode,
    pub address_mode_u: SamplerAddressMode,
    pub address_mode_v: SamplerAddressMode,
    pub address_mode_w: SamplerAddressMode,
    pub mip_lod_bias: f32,
    pub anisotropy_enable: bool,
    pub max_anisotropy: f32,
    pub compare_enable: bool,
    pub compare_op: CompareOp,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: BorderColor,
    pub unnormalized_coordinates: bool,
}

pub struct Sampler {
    device: Rc<Device>,
    handle: ffi::Sampler,
}

impl Sampler {
    pub fn new(device: Rc<Device>, create_info: SamplerCreateInfo) -> Result<Self, Error> {
        let create_info = ffi::SamplerCreateInfo {
            structure_type: ffi::StructureType::SamplerCreateInfo,
            p_next: ptr::null(),
            flags: 0,
            mag_filter: create_info.mag_filter.into(),
            min_filter: create_info.min_filter.into(),
            mipmap_mode: create_info.mipmap_mode.into(),
            address_mode_u: create_info.address_mode_u.into(),
            address_mode_v: create_info.address_mode_v.into(),
            address_mode_w: create_info.address_mode_w.into(),
            mip_lod_bias: create_info.mip_lod_bias as _,
            anisotropy_enable: create_info.anisotropy_enable as _,
            max_anisotropy: create_info.max_anisotropy as _,
            compare_enable: create_info.compare_enable as _,
            compare_op: create_info.compare_op.into(),
            min_lod: create_info.min_lod as _,
            max_lod: create_info.max_lod as _,
            border_color: create_info.border_color.into(),
            unnormalized_coordinates: create_info.unnormalized_coordinates.into(),
        };

        let mut handle = MaybeUninit::<ffi::Sampler>::uninit();

        let result = unsafe {
            ffi::vkCreateSampler(
                device.handle,
                &create_info,
                ptr::null(),
                handle.as_mut_ptr(),
            )
        };

        match result {
            ffi::Result::Success => {
                let handle = unsafe { handle.assume_init() };

                let sampler = Self { device, handle };

                Ok(sampler)
            }
            ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
            ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
            _ => panic!("unexpected result: {:?}", result),
        }
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroySampler(self.device.handle, self.handle, ptr::null()) };
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

#[derive(Clone, Copy)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    IncrementAndClamp,
    DecrementAndClamp,
    Invert,
    IncrementAndWrap,
    DecrementAndWrap,
}

pub struct ImageMemoryBarrier<'a> {
    pub src_access_mask: u32,
    pub dst_access_mask: u32,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub image: &'a Image,
    pub subresource_range: ImageSubresourceRange,
}

pub struct MemoryBarrier {
    pub src_access_mask: u32,
    pub dst_access_mask: u32,
}

pub struct BufferMemoryBarrier<'a> {
    pub src_access_mask: u32,
    pub dst_access_mask: u32,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub buffer: &'a Buffer,
    pub offset: u64,
    pub size: u64,
}
