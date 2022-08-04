use crate::prelude::*;

pub struct RenderPassInfo<'a> {
    device: &'a Device,
    attachments: &'a [Attachment],
    subpasses: &'a [Subpass<'a>],
}

pub struct Subpass<'a> {
    src: Option<u32>,
    src_access: Access,
    src_stage: PipelineStage,
    dst: Option<u32>,
    dst_access: Access,
    dst_stage: PipelineStage,
    attachments: &'a [u32],
}

pub struct Attachment {
    format: Format,
    load_op: AttachmentLoadOp,
    store_op: AttachmentStoreOp,
    layout: ImageLayout,
    ty: AttachmentType,
}

#[derive(Clone, Copy)]
pub enum AttachmentLoadOp {
    DontCare,
    Load,
    Clear,
}

impl From<AttachmentLoadOp> for vk::AttachmentLoadOp {
    fn from(op: AttachmentLoadOp) -> Self {
        match op {
            AttachmentLoadOp::DontCare => Self::DontCare,
            AttachmentLoadOp::Load => Self::Load,
            AttachmentLoadOp::Clear => Self::Clear,
        }
    }
}

#[derive(Clone, Copy)]
pub enum AttachmentStoreOp {
    DontCare,
    Store,
}

impl From<AttachmentStoreOp> for vk::AttachmentStoreOp {
    fn from(op: AttachmentStoreOp) -> Self {
        match op {
            AttachmentStoreOp::DontCare => Self::DontCare,
            AttachmentStoreOp::Store => Self::Store,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AttachmentType {
    Color,
    DepthStencil,
    Input,
}

pub enum RenderPass {
    Vulkan { render_pass: vk::RenderPass },
}

impl RenderPass {
    pub fn new(info: RenderPassInfo<'_>) -> Self {
        match info.device {
            Device::Vulkan { device, .. } => {
                let attachments = info
                    .attachments
                    .iter()
                    .map(|attachment| vk::AttachmentDescription {
                        format: attachment.format.into(),
                        samples: vk::SAMPLE_COUNT_1,
                        load_op: attachment.load_op.into(),
                        store_op: attachment.store_op.into(),
                        stencil_load_op: vk::AttachmentLoadOp::DontCare,
                        stencil_store_op: vk::AttachmentStoreOp::DontCare,
                        initial_layout: vk::ImageLayout::Undefined,
                        final_layout: attachment.layout.into(),
                    })
                    .collect::<Vec<_>>();

                let input_attachments = info
                    .subpasses
                    .iter()
                    .map(|subpass| {
                        subpass
                            .attachments
                            .iter()
                            .filter(|&attachment| {
                                info.attachments[*attachment as usize].ty == AttachmentType::Input
                            })
                            .map(|&attachment| vk::AttachmentReference {
                                attachment,
                                layout: info.attachments[attachment as usize].layout.into(),
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                let color_attachments = info
                    .subpasses
                    .iter()
                    .map(|subpass| {
                        subpass
                            .attachments
                            .iter()
                            .filter(|&attachment| {
                                info.attachments[*attachment as usize].ty == AttachmentType::Color
                            })
                            .map(|&attachment| vk::AttachmentReference {
                                attachment,
                                layout: info.attachments[attachment as usize].layout.into(),
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                let depth_stencil_attachments = info
                    .subpasses
                    .iter()
                    .map(|subpass| {
                        subpass
                            .attachments
                            .iter()
                            .filter(|&attachment| {
                                info.attachments[*attachment as usize].ty
                                    == AttachmentType::DepthStencil
                            })
                            .map(|&attachment| vk::AttachmentReference {
                                attachment,
                                layout: info.attachments[attachment as usize].layout.into(),
                            })
                            .take(1)
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                let subpasses = (0..info.subpasses.len())
                    .map(|i| vk::SubpassDescription {
                        pipeline_bind_point: vk::PipelineBindPoint::Graphics,
                        input_attachments: &input_attachments[i],
                        color_attachments: &color_attachments[i],
                        resolve_attachments: &[],
                        depth_stencil_attachment: depth_stencil_attachments[i].get(0),
                        preserve_attachments: &[],
                    })
                    .collect::<Vec<_>>();

                let dependencies = info
                    .subpasses
                    .iter()
                    .map(|subpass| vk::SubpassDependency {
                        src_subpass: subpass.src.unwrap_or(vk::SUBPASS_EXTERNAL),
                        dst_subpass: subpass.dst.unwrap_or(vk::SUBPASS_EXTERNAL),
                        src_stage_mask: subpass.src_stage.to_vk(),
                        src_access_mask: subpass.src_access.to_vk(),
                        dst_stage_mask: subpass.dst_stage.to_vk(),
                        dst_access_mask: subpass.dst_access.to_vk(),
                    })
                    .collect::<Vec<_>>();

                let render_pass_create_info = vk::RenderPassCreateInfo {
                    attachments: &attachments,
                    subpasses: &subpasses,
                    dependencies: &dependencies,
                };

                let render_pass = vk::RenderPass::new(device.clone(), render_pass_create_info)
                    .expect("failed to create render pass");

                Self::Vulkan { render_pass }
            }
        }
    }
}
