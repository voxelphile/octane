use std::ffi::{CStr, CString};
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::rc::Rc;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

mod ffi {
    use std::ffi::{CStr, CString};
    use std::mem;

    use libc::{c_char, c_float, c_int, c_uint, c_ulong, c_void, size_t};

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
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub enum StructureType {
        ApplicationInfo = 0,
        InstanceCreateInfo = 1,
        DeviceQueueCreateInfo = 2,
        DeviceCreateInfo = 3,
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
    pub struct Instance(*mut u8);

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PhysicalDevice(*mut u8);

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct Device(*mut u8);

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct Queue(*mut u8);

    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct DebugUtilsMessenger(u64);

    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct Surface(u64);

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
    }
}

pub const KHR_SURFACE: &str = "VK_KHR_surface";
pub const KHR_XLIB_SURFACE: &str = "VK_KHR_xlib_surface";

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

pub type DebugUtilsMessengerCallback = fn(&DebugUtilsMessengerCallbackData) -> bool;

#[derive(Clone, Copy, Debug)]
pub enum Error {
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
        physical_device: PhysicalDevice,
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
