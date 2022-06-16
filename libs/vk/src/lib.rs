use std::ffi::{CStr, CString};
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::rc::Rc;

mod ffi {
    use std::ffi::{CStr, CString};
    use std::mem;

    use libc::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

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
        DebugUtilsMessengerCreateInfo = 1000128004,
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
    pub struct Instance(*mut u8);

    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct DebugUtilsMessenger(u64);

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
