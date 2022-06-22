use std::ffi::{CStr, CString};
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::rc::Rc;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

mod ffi {
    use std::ffi::{CStr, CString};
    use std::fmt;
    use std::mem;

    use libc::{c_char, c_float, c_int, c_uint, c_ulong, c_void, size_t};

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

    handle_nondispatchable!(DebugUtilsMessenger);
    handle_nondispatchable!(Surface);
    handle_nondispatchable!(Swapchain);
    handle_nondispatchable!(Image);
    handle_nondispatchable!(ImageView);
    handle_nondispatchable!(ShaderModule);

    #[derive(Clone, Copy, Debug)]
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
        SurfaceLost = -1000000000,
        NativeWindowInUse = -1000000001,
        InvalidShader = -1000012000,
        CompressionExhausted = -1000338000,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum StructureType {
        ApplicationInfo = 0,
        InstanceCreateInfo = 1,
        DeviceQueueCreateInfo = 2,
        DeviceCreateInfo = 3,
        ImageViewCreateInfo = 15,
        ShaderModuleCreateInfo = 16,
        PipelineShaderStageCreateInfo = 18,
        SwapchainCreateInfo = 1000001000,
        XlibSurfaceCreateInfo = 1000004000,
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
        Bgra8Srgb = 50,
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

    pub type Extent2d = [c_uint; 2];
    pub type Extent3d = [c_uint; 3];

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SurfaceCapabilities {
        pub min_image_count: c_uint,
        pub max_image_count: c_uint,
        pub current_extent: Extent2d,
        pub min_image_extent: Extent2d,
        pub max_image_extent: Extent2d,
        pub max_image_array_layers: c_uint,
        pub supported_transforms: c_uint,
        pub current_transform: c_uint,
        pub supported_composite_alpha: c_uint,
        pub supported_usage_flags: c_uint,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SurfaceFormat {
        pub format: Format,
        pub color_space: ColorSpace,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum ImageUsage {
        ColorAttachment = 0x00000010,
        DepthStencilAttachment = 0x00000020,
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
        pub p_next: *const c_void,
        pub application_name: *const c_char,
        pub application_version: c_uint,
        pub engine_name: *const c_char,
        pub engine_version: c_uint,
        pub api_version: c_uint,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct InstanceCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const c_void,
        pub flags: c_uint,
        pub application_info: *const ApplicationInfo,
        pub enabled_layer_count: c_uint,
        pub enabled_layer_names: *const *const c_char,
        pub enabled_extension_count: c_uint,
        pub enabled_extension_names: *const *const c_char,
    }
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DebugUtilsLabel {
        pub structure_type: StructureType,
        pub p_next: *const c_void,
        pub label_name: *const c_char,
        pub color: [f32; 4],
    }
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DebugUtilsObjectNameInfo {
        pub structure_type: StructureType,
        pub p_next: *const c_void,
        pub object_type: ObjectType,
        pub object_handle: c_ulong,
        pub object_name: *const c_char,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DebugUtilsMessengerCallbackData {
        pub structure_type: StructureType,
        pub p_next: *const c_void,
        pub flags: c_uint,
        pub message_id_name: *const c_char,
        pub message_id_number: c_int,
        pub message: *const c_char,
        pub queue_label_count: c_uint,
        pub queue_labels: *const DebugUtilsLabel,
        pub cmd_buf_label_count: c_uint,
        pub cmd_buf_labels: *const DebugUtilsLabel,
        pub object_count: c_uint,
        pub objects: *const DebugUtilsObjectNameInfo,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DebugUtilsMessengerCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const c_void,
        pub flags: c_uint,
        pub message_severity: c_int,
        pub message_type: c_int,
        pub user_callback: DebugUtilsMessengerCallbackInternal,
        pub user_data: *const c_void,
    }

    pub type DebugUtilsMessengerCallbackInternal = unsafe extern "system" fn(
        c_uint,
        c_uint,
        *const DebugUtilsMessengerCallbackData,
        *const c_void,
    ) -> bool;

    pub type CreateDebugUtilsMessenger = unsafe extern "system" fn(
        Instance,
        *const DebugUtilsMessengerCreateInfo,
        *const c_void,
        *mut DebugUtilsMessenger,
    ) -> Result;

    pub type DestroyDebugUtilsMessenger =
        unsafe extern "system" fn(Instance, DebugUtilsMessenger, *const c_void) -> Result;

    pub unsafe extern "system" fn debug_utils_messenger_callback(
        message_severity: c_uint,
        message_type: c_uint,
        callback_data: *const DebugUtilsMessengerCallbackData,
        user_data: *const c_void,
    ) -> bool {
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

        f(&exposed_callback_data)
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
        pub max_image_dimension_1d: c_uint,
        pub max_image_dimension_2d: c_uint,
        pub max_image_dimension_3d: c_uint,
        pub max_image_dimension_cube: c_uint,
        pub max_image_array_layers: c_uint,
        pub max_texel_buffer_elements: c_uint,
        pub max_uniform_buffer_range: c_uint,
        pub max_storage_buffer_range: c_uint,
        pub max_push_constants_size: c_uint,
        pub max_memory_allocation_count: c_uint,
        pub max_sampler_allocation_count: c_uint,
        pub buffer_image_granularity: c_ulong,
        pub sparse_address_space_size: c_ulong,
        pub max_bound_descriptor_sets: c_uint,
        pub max_per_stage_descriptor_samplers: c_uint,
        pub max_per_stage_descriptor_uniform_buffers: c_uint,
        pub max_per_stage_descriptor_storage_buffers: c_uint,
        pub max_per_stage_descriptor_sampled_images: c_uint,
        pub max_per_stage_descriptor_storage_images: c_uint,
        pub max_per_stage_descriptor_input_attachments: c_uint,
        pub max_per_stage_resources: c_uint,
        pub max_descriptor_set_samplers: c_uint,
        pub max_descriptor_set_uniform_buffers: c_uint,
        pub max_descriptor_set_uniform_buffers_dynamic: c_uint,
        pub max_descriptor_set_storage_buffers: c_uint,
        pub max_descriptor_set_storage_buffers_dynamic: c_uint,
        pub max_descriptor_set_sampled_images: c_uint,
        pub max_descriptor_set_storage_images: c_uint,
        pub max_descriptor_set_input_attachments: c_uint,
        pub max_vertex_input_attributes: c_uint,
        pub max_vertex_input_bindings: c_uint,
        pub max_vertex_input_binding_stride: c_uint,
        pub max_vertex_output_components: c_uint,
        pub max_tessellation_generation_level: c_uint,
        pub max_tessellation_patch_size: c_uint,
        pub max_tessellation_control_per_vertex_input_components: c_uint,
        pub max_tessellation_control_per_vertex_output_components: c_uint,
        pub max_tessellation_control_per_patch_output_components: c_uint,
        pub max_tessellation_total_output_components: c_uint,
        pub max_tessellation_evaluation_input_components: c_uint,
        pub max_tessellation_evaluation_output_components: c_uint,
        pub max_geometry_shader_invocations: c_uint,
        pub max_geometry_input_components: c_uint,
        pub max_geometry_output_components: c_uint,
        pub max_geometry_output_vertices: c_uint,
        pub max_geometry_total_output_components: c_uint,
        pub max_fragment_input_components: c_uint,
        pub max_fragment_output_attachments: c_uint,
        pub max_fragment_dual_src_attachments: c_uint,
        pub max_fragment_combined_output_resources: c_uint,
        pub max_compute_shared_memory_size: c_uint,
        pub max_compute_work_group_count: [c_uint; 3],
        pub max_compute_work_group_invocations: c_uint,
        pub max_compute_work_group_size: [c_uint; 3],
        pub sub_pixel_precision_bits: c_uint,
        pub sub_texel_precision_bits: c_uint,
        pub mipmap_precision_bits: c_uint,
        pub max_draw_indexed_index_value: c_uint,
        pub max_draw_indirect_count: c_uint,
        pub max_sampler_lod_bias: c_float,
        pub max_sampler_anisotropy: c_float,
        pub max_viewports: c_uint,
        pub max_viewport_dimensions: [c_uint; 2],
        pub viewport_bounds_range: [c_float; 2],
        pub viewport_sub_pixel_bits: c_uint,
        pub min_memory_map_alignment: size_t,
        pub min_texel_buffer_offset_alignment: c_ulong,
        pub min_uniform_buffer_offset_alignment: c_ulong,
        pub min_storage_buffer_offset_alignment: c_ulong,
        pub min_texel_offset: c_int,
        pub max_texel_offset: c_uint,
        pub min_texel_gather_offset: c_int,
        pub max_texel_gather_offset: c_uint,
        pub min_interpolation_offset: c_float,
        pub max_interpolation_offset: c_float,
        pub sub_pixel_interpolation_offset_bits: c_uint,
        pub max_framebuffer_width: c_uint,
        pub min_framebuffer_width: c_uint,
        pub min_framebuffer_layers: c_uint,
        pub framebuffer_color_sample_counts: c_uint,
        pub framebuffer_depth_sample_counts: c_uint,
        pub framebuffer_stencil_sample_counts: c_uint,
        pub framebuffer_no_attachments_sample_counts: c_uint,
        pub max_color_attachments: c_uint,
        pub sampled_image_color_sample_counts: c_uint,
        pub sampled_image_integer_sample_counts: c_uint,
        pub sampled_image_depth_sample_counts: c_uint,
        pub sampled_image_stencil_sample_counts: c_uint,
        pub storage_image_sample_counts: c_uint,
        pub max_sample_mask_words: c_uint,
        pub timestamp_compute_and_graphics: bool,
        pub timestamp_period: c_float,
        pub max_clip_distances: c_uint,
        pub max_cull_distances: c_uint,
        pub max_combined_clip_and_cull_distances: c_uint,
        pub discrete_queue_priorities: c_uint,
        pub point_size_range: [c_float; 2],
        pub line_width_range: [c_float; 2],
        pub point_size_granularity: c_float,
        pub line_width_granularity: c_float,
        pub strict_lines: bool,
        pub standard_sample_locations: bool,
        pub optimal_buffer_copy_offset_alignment: c_uint,
        pub optimal_buffer_copy_row_pitch_alignment: c_uint,
        pub non_coherent_atom_size: c_uint,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PhysicalDeviceSparseProperties {
        pub residency_standard_2d_block_shape: bool,
        pub residency_standard_2d_multisample_block_shape: bool,
        pub residency_standard_3d_block_shape: bool,
        pub residency_aligned_mip_size: bool,
        pub residency_non_resident_strict: bool,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PhysicalDeviceProperties {
        pub api_version: c_uint,
        pub driver_version: c_uint,
        pub vendor_id: c_uint,
        pub device_id: c_uint,
        pub device_type: PhysicalDeviceType,
        pub device_name: [c_char; 256],
        pub pipeline_cache_uuid: [c_char; 16],
        pub limits: PhysicalDeviceLimits,
        pub sparse_properties: PhysicalDeviceSparseProperties,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct QueueFamilyProperties {
        pub queue_flags: c_uint,
        pub queue_count: c_uint,
        pub timestamp_valid_bits: c_uint,
        pub min_image_transfer_granularity: [c_uint; 3],
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DeviceQueueCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const c_void,
        pub flags: c_uint,
        pub queue_family_index: c_uint,
        pub queue_count: c_uint,
        pub queue_priorities: *const c_float,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct DeviceCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const c_void,
        pub flags: c_uint,
        pub queue_create_info_count: c_uint,
        pub queue_create_infos: *const DeviceQueueCreateInfo,
        pub enabled_layer_count: c_uint,
        pub enabled_layer_names: *const *const c_char,
        pub enabled_extension_count: c_uint,
        pub enabled_extension_names: *const *const c_char,
        pub enabled_features: *const c_void,
    }

    #[cfg(target_os = "linux")]
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct XlibSurfaceCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const c_void,
        pub flags: c_uint,
        pub display: *const c_void,
        pub window: c_ulong,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct SwapchainCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const c_void,
        pub flags: c_uint,
        pub surface: Surface,
        pub min_image_count: c_uint,
        pub image_format: Format,
        pub image_color_space: ColorSpace,
        pub image_extent: Extent2d,
        pub image_array_layers: c_uint,
        pub image_usage: ImageUsage,
        pub image_sharing_mode: SharingMode,
        pub queue_family_index_count: c_uint,
        pub queue_family_indices: *const c_uint,
        pub pre_transform: c_uint,
        pub composite_alpha: CompositeAlpha,
        pub present_mode: PresentMode,
        pub clipped: bool,
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
        pub aspect_mask: c_uint,
        pub base_mip_level: c_uint,
        pub level_count: c_uint,
        pub base_array_layer: c_uint,
        pub layer_count: c_uint,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ImageViewCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const c_void,
        pub flags: c_uint,
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
        pub p_next: *const c_void,
        pub flags: c_uint,
        pub code_size: size_t,
        pub code: *const c_uint,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum ShaderStage {
        Vertex = 0x00000008,
        Fragment = 0x00000080,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PipelineShaderStageCreateInfo {
        pub structure_type: StructureType,
        pub p_next: *const c_void,
        pub flags: c_uint,
        pub stage: ShaderStage,
        pub module: ShaderModule,
        pub entry_point: *const c_char,
        pub specialization_info: *const c_void,
    }

    #[link(name = "vulkan")]
    #[allow(non_snake_case)]
    extern "C" {
        //TODO implement VkAllocationCallbacks
        pub fn vkCreateInstance(
            create_info: *const InstanceCreateInfo,
            allocator: *const c_void,
            instance: *mut Instance,
        ) -> Result;
        pub fn vkDestroyInstance(instance: Instance, allocator: *const c_void);
        pub fn vkGetInstanceProcAddr(instance: Instance, name: *const c_char) -> *const c_void;
        pub fn vkEnumeratePhysicalDevices(
            instance: Instance,
            physical_device_count: *mut c_uint,
            physical_devices: *mut PhysicalDevice,
        ) -> Result;
        pub fn vkGetPhysicalDeviceProperties(
            physical_device: PhysicalDevice,
            properties: *mut PhysicalDeviceProperties,
        );
        pub fn vkGetPhysicalDeviceQueueFamilyProperties(
            physical_device: PhysicalDevice,
            queue_family_property_count: *mut c_uint,
            queue_family_properties: *mut QueueFamilyProperties,
        );
        pub fn vkGetPhysicalDeviceSurfaceCapabilitiesKHR(
            physical_device: PhysicalDevice,
            surface: Surface,
            surface_capabilities: *mut SurfaceCapabilities,
        );
        pub fn vkCreateDevice(
            physical_device: PhysicalDevice,
            create_info: *const DeviceCreateInfo,
            allocator: *const c_void,
            device: *mut Device,
        ) -> Result;
        pub fn vkDestroyDevice(device: Device, allocator: *const c_void);
        pub fn vkGetDeviceQueue(
            device: Device,
            queue_family_index: c_uint,
            queue_index: c_uint,
            queue: *mut Queue,
        );
        #[cfg(target_os = "linux")]
        pub fn vkCreateXlibSurfaceKHR(
            instance: Instance,
            create_info: *const XlibSurfaceCreateInfo,
            allocator: *const c_void,
            surface: *mut Surface,
        );
        pub fn vkDestroySurfaceKHR(instance: Instance, surface: Surface, allocator: *const c_void);
        pub fn vkCreateSwapchainKHR(
            device: Device,
            create_info: *const SwapchainCreateInfo,
            allocator: *const c_void,
            swapchain: *mut Swapchain,
        ) -> Result;
        pub fn vkDestroySwapchainKHR(
            device: Device,
            swapchain: Swapchain,
            allocator: *const c_void,
        );
        pub fn vkGetSwapchainImagesKHR(
            device: Device,
            swapchain: Swapchain,
            swapchain_image_count: *mut c_uint,
            swapchain_images: *mut Image,
        );
        pub fn vkCreateImageView(
            device: Device,
            create_info: *const ImageViewCreateInfo,
            allocator: *const c_void,
            image_view: *mut ImageView,
        ) -> Result;
        pub fn vkDestroyImageView(device: Device, image_view: ImageView, allocator: *const c_void);
        pub fn vkCreateShaderModule(
            device: Device,
            create_info: *const ShaderModuleCreateInfo,
            allocator: *const c_void,
            shader_module: *mut ShaderModule,
        ) -> Result;
        pub fn vkDestroyShaderModule(
            device: Device,
            shader_module: ShaderModule,
            allocator: *const c_void,
        );
    }
}

pub const KHR_SURFACE: &str = "VK_KHR_surface";
pub const KHR_XLIB_SURFACE: &str = "VK_KHR_xlib_surface";
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

pub const IMAGE_ASPECT_COLOR: u32 = 0x00000001;

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
    SurfaceLost,
    NativeWindowInUse,
    InvalidShader,
    CompressionExhausted,
}

#[derive(Clone, Copy)]
pub enum Format {
    Bgra8Srgb,
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
pub enum ImageUsage {
    ColorAttachment,
    DepthStencilAttachment,
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
            application_name: application_name.as_c_str().as_ptr(),
            application_version,
            engine_name: engine_name.as_c_str().as_ptr(),
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

        let extension_names = create_info
            .extensions
            .iter()
            .map(|extension_name| CString::new(*extension_name).unwrap())
            .collect::<Vec<_>>();

        let enabled_extension_names = extension_names
            .iter()
            .map(|string| string.as_ptr())
            .collect::<Vec<_>>();

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

        let p_next = if let Some(create_info) = debug_utils {
            unsafe { mem::transmute::<_, _>(&create_info) }
        } else {
            ptr::null()
        };

        let create_info = ffi::InstanceCreateInfo {
            structure_type: ffi::StructureType::InstanceCreateInfo,
            p_next,
            flags: 0,
            application_info: &application_info,
            enabled_layer_count: create_info.layers.len() as _,
            enabled_layer_names: enabled_layer_names.as_ptr(),
            enabled_extension_count: create_info.extensions.len() as _,
            enabled_extension_names: enabled_extension_names.as_ptr(),
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
            _ => panic!("unexpected result"),
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
            _ => panic!("unexpected result"),
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

#[derive(PartialEq, Eq)]
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

//TODO add info
pub struct PhysicalDeviceFeatures {}

pub struct PhysicalDevice {
    handle: ffi::PhysicalDevice,
}

impl PhysicalDevice {
    pub fn enumerate(instance: Rc<Instance>) -> Vec<Self> {
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

        let physical_devices = handles.iter().map(|&handle| Self { handle }).collect();

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

        let device_name = properties
            .device_name
            .iter()
            .map(|&c| c as u8 as char)
            .collect::<String>();

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
        PhysicalDeviceFeatures {}
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
            enabled_features: ptr::null(),
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
            _ => panic!("unexpected result"),
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
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyDevice(self.handle, ptr::null()) };
    }
}

pub struct Queue {
    handle: ffi::Queue,
}

pub struct Surface {
    instance: Rc<Instance>,
    handle: ffi::Surface,
}

#[cfg(target_os = "linux")]
impl Surface {
    pub fn new(instance: Rc<Instance>, window: &impl HasRawWindowHandle) -> Self {
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

                Self { instance, handle }
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
    pub image_usage: ImageUsage,
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

        let image_usage = match create_info.image_usage {
            ImageUsage::ColorAttachment => ffi::ImageUsage::ColorAttachment,
            _ => unimplemented!(),
        };

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
            PresentMode::Mailbox => ffi::PresentMode::Mailbox,
            PresentMode::Fifo => ffi::PresentMode::Fifo,
            _ => unimplemented!(),
        };

        let old_swapchain = create_info
            .old_swapchain
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
            clipped: create_info.clipped,
            old_swapchain,
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
            _ => panic!("unexpected result"),
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
            .map(|handle| Image { handle })
            .collect::<Vec<_>>();

        swapchain_images
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroySwapchainKHR(self.device.handle, self.handle, ptr::null()) };
    }
}

pub struct Image {
    handle: ffi::Image,
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
        let view_type = match create_info.view_type {
            ImageViewType::OneDim => ffi::ImageViewType::OneDim,
            ImageViewType::TwoDim => ffi::ImageViewType::TwoDim,
            ImageViewType::ThreeDim => ffi::ImageViewType::ThreeDim,
            ImageViewType::Cube => ffi::ImageViewType::Cube,
            ImageViewType::OneDimArray => ffi::ImageViewType::OneDimArray,
            ImageViewType::TwoDimArray => ffi::ImageViewType::TwoDimArray,
            ImageViewType::ThreeDimArray => ffi::ImageViewType::ThreeDimArray,
        };

        let format = match create_info.format {
            Format::Bgra8Srgb => ffi::Format::Bgra8Srgb,
        };

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
            _ => panic!("unexpected result"),
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
            _ => panic!("unexpected result"),
        }
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe { ffi::vkDestroyShaderModule(self.device.handle, self.handle, ptr::null()) };
    }
}

pub enum ShaderStage {
    Vertex,
    Fragment,
}

pub struct PipelineShaderStageCreateInfo<'a> {
    pub stage: ShaderStage,
    pub module: &'a ShaderModule,
    pub entry_point: &'a str,
}
