#![feature(derive_default_enum)]

pub mod bucket;
pub mod input;
pub mod mesh;
pub mod octree;
pub mod render;
pub mod voxel;

pub mod prelude {
    pub use crate::mesh::Mesh;
    pub use crate::render::{Batch, Object, Renderer};
}
