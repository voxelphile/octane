use std::ffi::CString;
use std::mem::MaybeUninit;
use std::ptr;

mod ffi {
    pub use libc::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

    #[derive(Clone, Copy)]
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
        pub enabled_layer_names: *const c_char,
        pub enabled_extension_count: c_uint,
        pub enabled_extension_names: *const c_char,
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct Instance {
        opaque: [u8; 0],
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
    }
}

pub const KHR_SURFACE: &str = "VK_KHR_surface";
pub const KHR_XLIB_SURFACE: &str = "VK_KHR_xlib_surface";

pub const EXT_DEBUG_REPORT: &str = "VK_EXT_debug_report";

pub const LAYER_LUNARG_STANDARD_VALIDATION: &str = "VK_LAYER_LUNARG_standard_validation";

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

pub type Instance = ffi::Instance;

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
}

pub fn create_instance(create_info: InstanceCreateInfo<'_>) -> Result<Instance, Error> {
    let internal_application_name =
        CString::new(create_info.application_info.application_name).unwrap();

    let internal_application_version: u32 = create_info.application_info.application_version.into();

    let internal_engine_name = CString::new(create_info.application_info.application_name).unwrap();

    let internal_engine_version: u32 = create_info.application_info.engine_version.into();

    let internal_api_version: u32 = create_info.application_info.api_version.into();

    let internal_application_info = ffi::ApplicationInfo {
        structure_type: ffi::StructureType::ApplicationInfo,
        p_next: ptr::null(),
        application_name: internal_application_name.as_c_str().as_ptr(),
        application_version: internal_application_version,
        engine_name: internal_engine_name.as_c_str().as_ptr(),
        engine_version: internal_engine_version,
        api_version: internal_api_version,
    };

    let internal_layer_names = create_info
        .layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect::<Vec<_>>();

    let internal_layer_names = internal_layer_names
        .iter()
        .flat_map(|string| string.as_bytes_with_nul().iter().map(|byte| *byte as _))
        .collect::<Vec<_>>();

    let internal_extension_names = create_info
        .extensions
        .iter()
        .map(|extension_name| CString::new(*extension_name).unwrap())
        .collect::<Vec<_>>();

    let internal_extension_names = internal_extension_names
        .iter()
        .flat_map(|string| string.as_bytes_with_nul().iter().map(|byte| *byte as _))
        .collect::<Vec<_>>();

    let internal_create_info = ffi::InstanceCreateInfo {
        structure_type: ffi::StructureType::InstanceCreateInfo,
        p_next: ptr::null(),
        flags: 0,
        application_info: &internal_application_info,
        enabled_extension_names: internal_extension_names.as_ptr(),
        enabled_extension_count: 0,
        enabled_layer_names: internal_layer_names.as_ptr(),
        enabled_layer_count: 0,
    };

    let mut internal_instance = MaybeUninit::<ffi::Instance>::uninit();

    let result = unsafe {
        ffi::vkCreateInstance(
            &internal_create_info,
            ptr::null(),
            internal_instance.as_mut_ptr(),
        )
    };

    let internal_instance = unsafe { internal_instance.assume_init() };

    match result {
        ffi::Result::Success => Ok(internal_instance),
        ffi::Result::OutOfHostMemory => Err(Error::OutOfHostMemory),
        ffi::Result::OutOfDeviceMemory => Err(Error::OutOfDeviceMemory),
        ffi::Result::InitializationFailed => Err(Error::InitializationFailed),
        ffi::Result::LayerNotPresent => Err(Error::LayerNotPresent),
        ffi::Result::ExtensionNotPresent => Err(Error::ExtensionNotPresent),
        ffi::Result::IncompatibleDriver => Err(Error::IncompatibleDriver),
        _ => panic!("unexpected result"),
    }
}
