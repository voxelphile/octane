use bitflags::bitflags;

bitflags! {
    pub struct MemoryProperties: usize {
        const DEVICE_LOCAL = 1 << 0;
        const HOST_VISIBLE = 1 << 1;
        const HOST_COHERENT = 1 << 2;
    }
}

impl MemoryProperties {
    pub(crate) fn to_vk(self) -> u32 {
        let mut vk = 0;

        if self == Self::DEVICE_LOCAL {
            vk |= vk::MEMORY_PROPERTY_DEVICE_LOCAL;
        }

        if self == Self::HOST_VISIBLE {
            vk |= vk::MEMORY_PROPERTY_HOST_VISIBLE;
        }

        if self == Self::HOST_COHERENT {
            vk |= vk::MEMORY_PROPERTY_HOST_COHERENT;
        }

        vk
    }
}
