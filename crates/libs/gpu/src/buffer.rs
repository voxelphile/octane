use crate::prelude::*;

use bitflags::bitflags;

bitflags! {
    pub struct BufferUsage: usize {
        const TRANSFER_SRC  = 1 << 0;
        const TRANSFER_DST  = 1 << 1;
        const STORAGE       = 1 << 2;
        const UNIFORM       = 1 << 3;
        const VERTEX        = 1 << 4;
        const INDEX         = 1 << 5;
    }
}

impl BufferUsage {
    pub(crate) fn to_vk(self) -> u32 {
        let mut vk = 0;

        if self.contains(Self::TRANSFER_SRC) {
            vk |= vk::BUFFER_USAGE_TRANSFER_SRC;
        }

        if self.contains(Self::TRANSFER_DST) {
            vk |= vk::BUFFER_USAGE_TRANSFER_DST
        }

        if self.contains(Self::STORAGE) {
            vk |= vk::BUFFER_USAGE_STORAGE
        }

        if self.contains(Self::UNIFORM) {
            vk |= vk::BUFFER_USAGE_UNIFORM
        }

        if self.contains(Self::VERTEX) {
            vk |= vk::BUFFER_USAGE_VERTEX
        }

        if self.contains(Self::INDEX) {
            vk |= vk::BUFFER_USAGE_INDEX
        }

        vk
    }
}

pub struct BufferCopy<'a> {
    pub from: &'a Buffer,
    pub to: &'a mut Buffer,
    pub src: u64,
    pub dst: u64,
    pub size: u64,
}

pub struct BufferImageCopy<'a> {
    pub from: &'a Buffer,
    pub to: &'a mut Image,
    pub src: u64,
    pub dst_extent: (u32, u32, u32),
    pub dst_offset: (i32, i32, i32),
}

pub struct BufferWrite<'a, U: ?Sized + Copy> {
    pub offset: u64,
    pub data: &'a [U],
}

pub struct BufferInfo<'a> {
    pub device: &'a Device,
    pub usage: BufferUsage,
    pub properties: MemoryProperties,
    pub size: usize,
}

#[non_exhaustive]
pub enum Buffer {
    Vulkan {
        buffer: vk::Buffer,
        memory: vk::Memory,
    },
}

impl Buffer {
    pub fn new(info: BufferInfo) -> Self {
        match info.device {
            Device::Vulkan {
                physical_device,
                device,
                ..
            } => {
                let mut buffer =
                    vk::Buffer::new(device.clone(), info.size as u64, info.usage.to_vk())
                        .expect("failed to create buffer");

                let memory_allocate_info = vk::MemoryAllocateInfo {
                    property_flags: info.properties.to_vk(),
                };

                let memory = vk::Memory::allocate(
                    device.clone(),
                    memory_allocate_info,
                    buffer.memory_requirements(),
                    physical_device.memory_properties(),
                    info.properties.contains(MemoryProperties::HOST_VISIBLE),
                )
                .expect("failed to allocate memory");

                buffer.bind_memory(&memory);

                Self::Vulkan { buffer, memory }
            }
        }
    }

    pub fn write<U: ?Sized + Copy>(&mut self, write: BufferWrite<U>) {
        match self {
            Self::Vulkan { memory, .. } => {
                memory
                    .write(write.offset as _, |slice: &mut [U]| {
                        slice[..write.data.len()].copy_from_slice(write.data);
                    })
                    .expect("failed to write to buffer memory");
            }
        }
    }
}

pub enum Inner {
    Vulkan {},
}
