use crate::octree::*;
use crate::voxel::*;

use std::mem;
use std::fmt;

pub struct Bitfield {
    size: u32,
    data: Vec<u32>,
}

impl Bitfield {
    pub fn new(size: u32, data: Vec<u32>) -> Self {
        Self {
            size,
            data
        }
    }

    pub fn data(&self) -> &'_ [u32] {
        &self.data[..]
    }   
}

pub trait BitfieldBuilder {
    fn build_bitfield(&self) -> Bitfield;
}

impl BitfieldBuilder for SparseOctree<Voxel> {
    fn build_bitfield(&self) -> Bitfield {
        let size = self.size() as u32;
        
        let mut bits = 0;

        for i in 0..size {
            bits += 8usize.pow(i);
        }
        bits += 8usize.pow(size);

        let bit_size = ((bits as f32) / (8 * mem::size_of::<u32>()) as f32).ceil() as usize;

        let mut data = vec![0; bit_size];

        let mut todo = vec![0];

        while todo.len() > 0 {
            let node_index = todo.remove(0);
            
            let node = self.get_node_by_index(node_index).unwrap(); 

            if !node.voxel().is_transparent() {
                let morton = node.morton();
                
                let n = 8 * mem::size_of_val(&morton);

                let significance = morton.leading_zeros() as usize;

                let z = (n - significance) / 3 - 1;
                dbg!(z); 
                let mut space = 0;
                if n != significance { 
                    for j in 0..z {
                        space += 8usize.pow(j as u32);
                    }
                } else {
                    space = 0;
                }

                let morton = morton << significance + 3 >> significance + 3;

                let morton = morton + space as u64;
                dbg!(morton);

                let m = 8 * mem::size_of_val(&data[0]);

                data[morton as usize / m] |= 1 << (morton as usize % m);
            }

            if node.child() == u32::MAX {
                continue;
            }

            for i in 0..=7 {
                let mask = 1 << i;

                if node.valid() & mask != 0 {
                    let p = (node.valid() & (mask as u32 - 1)).count_ones();

                    let index = node.child() + p;

                    todo.push(index as _);
                }
            }
        }

        Bitfield::new(size, data)
    }    
}

impl fmt::Debug for Bitfield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[\n")?;

        let mut accum = 0;

        let m = 8 * mem::size_of_val(&self.data()[0]);

        let value = self.data[0] & 1;

        if value != 0 {
            write!(f, "homogenous\n")?;
        } else {
            write!(f, "different\n")?;
        }

        for i in 0..=self.size {
            let size = 8usize.pow(i as u32);

            let mut value = 0;
            let mut k = 0;

            for j in accum..accum + size {
                let x = j / m;
                let y = j % m;
                value <<= 1;
                value |= 1 & self.data[x] >> y;
                accum += 1;
                k += 1;
                if k == 8 {
                    write!(f, "{}", format!("{value:#?}\n"))?;
                    value = 0;
                    k = 0;
                }
            }
            if i != self.size {
                write!(f, "\n")?;
            }
        }

        write!(f, "]\n")
    }
}
