#![feature(let_chains)]

mod access;
mod buffer;
mod context;
mod device;
mod format;
mod framebuffer;
mod image;
mod memory;
mod pipeline;
mod render_pass;
mod shader;
mod surface;
mod swapchain;

pub mod prelude {
    pub use crate::access::*;
    pub use crate::buffer::*;
    pub use crate::context::*;
    pub use crate::device::*;
    pub use crate::format::*;
    pub use crate::framebuffer::*;
    pub use crate::image::*;
    pub use crate::memory::*;
    pub use crate::pipeline::*;
    pub use crate::render_pass::*;
    pub use crate::shader::*;
    pub use crate::surface::*;
    pub use crate::swapchain::*;
}
