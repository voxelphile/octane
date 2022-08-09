use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    pub struct Access: u32 {
        const SHADER_READ = 0x00000020;
        const COLOR_ATTACHMENT_READ = 0x00000080;
        const COLOR_ATTACHMENT_WRITE = 0x00000100;
        const DEPTH_STENCIL_ATTACHMENT_READ = 0x00000200;
        const DEPTH_STENCIL_ATTACHMENT_WRITE = 0x00000400;
    }
}

impl Access {
    pub fn to_vk(self) -> u32 {
        self.bits()
    }
}
