use crate::prelude::*;

pub struct FramebufferInfo<'a> {
    device: &'a Device,
    render_pass: &'a RenderPass,
    extent: (u32, u32, u32),
    attachments: &'a [Image],
}

pub enum Framebuffer {
    Vulkan { framebuffer: vk::Framebuffer },
}

impl Framebuffer {
    pub fn new(info: FramebufferInfo<'_>) -> Self {
        match info.device {
            Device::Vulkan { device, .. } => {
                let render_pass = if let RenderPass::Vulkan { render_pass } = info.render_pass {
                    render_pass
                } else {
                    panic!("not a vulkan surface");
                };

                let attachments = info
                    .attachments
                    .iter()
                    .map(|image| match image {
                        Image::Vulkan { view, .. } => view,
                        _ => panic!("not a vulkan image"),
                    })
                    .collect::<Vec<_>>();

                let framebuffer_create_info = vk::FramebufferCreateInfo {
                    render_pass: &render_pass,
                    attachments: &attachments,
                    width: info.extent.0,
                    height: info.extent.1,
                    layers: info.extent.2,
                };

                let framebuffer = vk::Framebuffer::new(device.clone(), framebuffer_create_info)
                    .expect("failed to create framebuffer");

                Self::Vulkan { framebuffer }
            }
        }
    }
}
