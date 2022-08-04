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

impl From<Format> for vk::Format {
    fn from(format: Format) -> Self {
        match format {
            Format::Rgba8Srgb => Self::Rgba8Srgb,
            Format::Bgra8Srgb => Self::Bgra8Srgb,
            Format::R16Uint => Self::R16Uint,
            Format::R32Uint => Self::R32Uint,
            Format::R32Sfloat => Self::R32Sfloat,
            Format::Rg32Sfloat => Self::Rg32Sfloat,
            Format::Rg32Sfloat => Self::Rg32Sfloat,
            Format::Rgb32Uint => Self::Rgb32Uint,
            Format::Rgb32Sfloat => Self::Rgb32Sfloat,
            Format::Rgba32Sfloat => Self::Rgba32Sfloat,
            Format::D32Sfloat => Self::D32Sfloat,
        }
    }
}

impl From<vk::Format> for Format {
    fn from(format: vk::Format) -> Self {
        match format {
            vk::Format::Rgba8Srgb => Self::Rgba8Srgb,
            vk::Format::Bgra8Srgb => Self::Bgra8Srgb,
            vk::Format::R16Uint => Self::R16Uint,
            vk::Format::R32Uint => Self::R32Uint,
            vk::Format::R32Sfloat => Self::R32Sfloat,
            vk::Format::Rg32Sfloat => Self::Rg32Sfloat,
            vk::Format::Rg32Sfloat => Self::Rg32Sfloat,
            vk::Format::Rgb32Uint => Self::Rgb32Uint,
            vk::Format::Rgb32Sfloat => Self::Rgb32Sfloat,
            vk::Format::Rgba32Sfloat => Self::Rgba32Sfloat,
            vk::Format::D32Sfloat => Self::D32Sfloat,
        }
    }
}
