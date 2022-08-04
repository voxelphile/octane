use crate::prelude::*;

use std::fs;
use std::io::prelude::*;
use std::mem;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    pub struct ShaderStage: u32 {
        const VERTEX = 0x00000001;
        const FRAGMENT = 0x00000010;
        const COMPUTE = 0x00000020;
    }
}

impl ShaderStage {
    pub(crate) fn to_vk(self) -> u32 {
        self.bits()
    }
}

#[derive(Debug, Clone)]
pub enum ShaderError {
    Compilation(u32, String),
    InvalidResource,
}

#[derive(Clone)]
pub enum ShaderInput {
    Spirv { asset: PathBuf },
    Glsl { asset: PathBuf, resource: PathBuf },
}

impl ShaderInput {
    pub(crate) fn get_resource(&self) -> Option<&Path> {
        match self {
            Self::Glsl { resource, .. } => Some(&resource),
            _ => None,
        }
    }

    pub(crate) fn get_asset(&self) -> &Path {
        match self {
            Self::Spirv { asset } => &asset,
            Self::Glsl { asset, .. } => &asset,
        }
    }
}

#[derive(Clone)]
pub struct ShaderLastModified {
    asset: SystemTime,
    resource: Option<SystemTime>,
}

impl ShaderLastModified {
    fn from_input(input: &ShaderInput) -> ShaderLastModified {
        ShaderLastModified {
            asset: {
                let metadata =
                    fs::metadata(input.get_asset()).expect("failed to get metadata of shader file");

                metadata
                    .modified()
                    .expect("failed to get last modified of shader file")
            },
            resource: input.get_resource().map(|resource| {
                let metadata =
                    fs::metadata(resource).expect("failed to get metadata of shader file");

                metadata
                    .modified()
                    .expect("failed to get last modified of shader file")
            }),
        }
    }
}

pub struct ShaderInfo<'a> {
    pub device: &'a Device,
    pub entry: &'a str,
    pub input: ShaderInput,
}

#[non_exhaustive]
pub enum Shader {
    Vulkan {
        device: Rc<vk::Device>,
        shader_module: vk::ShaderModule,
        entry: String,
        input: ShaderInput,
        last_modified: ShaderLastModified,
    },
}

impl Shader {
    pub fn new(info: ShaderInfo) -> Self {
        match info.device {
            Device::Vulkan { device, .. } => {
                let last_modified = ShaderLastModified::from_input(&info.input);

                if let Some(_) = info.input.get_resource() {
                    Self::compile_spirv(&info.input, &last_modified, &info.entry)
                        .expect("failed to compile shader");
                }

                let mut file =
                    fs::File::open(info.input.get_asset()).expect("failed to open shader file");

                let shader_module = Self::load_vk_shader(device.clone(), &mut file);

                Self::Vulkan {
                    device: device.clone(),
                    entry: info.entry.to_string(),
                    input: info.input,
                    last_modified,
                    shader_module,
                }
            }
        }
    }

    pub fn reload(&mut self) -> Result<bool, ShaderError> {
        match self {
            Self::Vulkan {
                shader_module,
                last_modified,
                device,
                input,
                entry,
            } => {
                if let Some(_) = input.get_resource() {
                    Self::compile_spirv(input, last_modified, entry)?;
                }

                let modified = {
                    let metadata = fs::metadata(input.get_asset())
                        .expect("failed to get metadata of shader file");

                    metadata
                        .modified()
                        .expect("failed to get last modified of shader file")
                };

                let reload = modified != last_modified.asset;

                if reload {
                    *last_modified = ShaderLastModified::from_input(&input);

                    let mut file =
                        fs::File::open(input.get_asset()).expect("failed to open shader file");

                    *shader_module = Self::load_vk_shader(device.clone(), &mut file);
                }

                Ok(reload)
            }
        }
    }

    fn compile_spirv(
        input: &ShaderInput,
        last_modified: &ShaderLastModified,
        entry: &'_ str,
    ) -> Result<(), ShaderError> {
        let asset = input.get_asset();

        let resource = input
            .get_resource()
            .expect("failed to get resource file path");

        let modified = {
            let metadata = fs::metadata(resource).expect("failed to get metadata of shader file");

            metadata
                .modified()
                .expect("failed to get last modified of shader file")
        };

        if modified == last_modified.resource.unwrap() {
            return Ok(());
        }

        let mut source_file = fs::File::open(resource).expect("failed to open shader file");

        let mut buffer = String::new();

        source_file
            .read_to_string(&mut buffer)
            .expect("failed to read shader from file");

        use shaderc::*;

        let kind = resource.file_stem().and_then(|stem| {
            let stem_str = stem.to_string_lossy();

            let stem_str_spl = stem_str.split(".").collect::<Vec<_>>();

            let ty = stem_str_spl[stem_str_spl.len() - 1];

            match ty {
                "vert" => Some(ShaderKind::Vertex),
                "frag" => Some(ShaderKind::Fragment),
                "comp" => Some(ShaderKind::Compute),
                _ => None,
            }
        });

        if let None = kind {
            return Err(ShaderError::InvalidResource);
        }

        let os_name = resource
            .file_name()
            .expect("failed to get shader file name");

        let name = os_name.to_str().unwrap();

        let kind = kind.unwrap();

        let compiler = Compiler::new().unwrap();

        let mut options = CompileOptions::new().unwrap();

        options.add_macro_definition("EP", Some(entry));

        let artifact = compiler
            .compile_into_spirv(&buffer, kind, name, entry, Some(&options))
            .map_err(|err| match err {
                Error::CompilationError(num, details) => ShaderError::Compilation(num, details),
                _ => panic!("failed to compile shader to spirv"),
            })?;

        let binary = artifact
            .as_binary()
            .iter()
            .flat_map(|a| a.to_le_bytes().into_iter())
            .collect::<Vec<_>>();

        if fs::metadata(&asset).is_ok() {
            fs::remove_file(&asset).expect("failed to remove file");
        }

        fs::write(&asset, &binary).expect("failed to write shader");

        Ok(())
    }

    fn load_vk_shader(device: Rc<vk::Device>, file: &mut fs::File) -> vk::ShaderModule {
        let mut bytes = vec![];

        file.read_to_end(&mut bytes)
            .expect("failed to read shader from file");

        let endian = mem::size_of::<u32>() / mem::size_of::<u8>();

        if bytes.len() % endian != 0 {
            panic!("cannot convert bytes to int; too few or too many")
        }

        let mut code = Vec::with_capacity(bytes.len() / endian);

        for slice in bytes.chunks(endian) {
            code.push(u32::from_le_bytes(slice.try_into().unwrap()));
        }

        let shader_module_create_info = vk::ShaderModuleCreateInfo { code: &code[..] };

        let shader_module = vk::ShaderModule::new(device, shader_module_create_info)
            .expect("failed to create shader module");

        shader_module
    }
}
