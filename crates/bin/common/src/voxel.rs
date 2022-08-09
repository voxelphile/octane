#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum Id {
    #[default]
    Error = 0,
    Vacuum = 42069,
    Air = 1,
    Grass = 2,
    Water = 3,
    Dirt = 4,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Voxel {
    pub id: Id,
}

impl Voxel {
    pub fn air() -> Self {
        Self { id: Id::Air }
    }

    pub fn is_translucent(&self) -> bool {
        match self.id {
            Id::Vacuum | Id::Air => true,
            _ => false,
        }
    }
}
