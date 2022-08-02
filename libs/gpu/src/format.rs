#[derive(Clone, Copy)]
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
    pub(crate) fn to_vk(self) -> vk::Format {
        match self {
            Self::Rgba8Srgb => vk::Format::Rgba8Srgb,
            Self::Bgra8Srgb => vk::Format::Bgra8Srgb,
            Self::R16Uint => vk::Format::R16Uint,
            Self::R32Uint => vk::Format::R32Uint,
            Self::R32Sfloat => vk::Format::R32Sfloat,
            Self::Rg32Sfloat => vk::Format::Rg32Sfloat,
            Self::Rg32Sfloat => vk::Format::Rg32Sfloat,
            Self::Rgb32Uint => vk::Format::Rgb32Uint,
            Self::Rgb32Sfloat => vk::Format::Rgb32Sfloat,
            Self::Rgba32Sfloat => vk::Format::Rgba32Sfloat,
            Self::D32Sfloat => vk::Format::D32Sfloat,
        }
    }
}
