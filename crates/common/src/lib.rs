pub mod mesh;
pub mod octree;
pub mod render;

pub mod prelude {
    pub use crate::mesh::Mesh;
    pub use crate::render::{Batch, Entry, Renderer};
}
