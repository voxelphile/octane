use crate::voxel::Voxel;

use math::prelude::Vector;

use std::marker;
use std::ops::RangeInclusive;

pub const PAGE_SIZE: usize = 4000;
pub trait Octree<T> {
    fn new() -> Self;
    fn place(&mut self, x: usize, y: usize, z: usize, nodes: T);
}
/*
pub struct BOctree {

}
*/

pub struct SparseOctree<T> {
    size: usize,
    node_count: Vec<RangeInclusive<usize>>,
    nodes: Vec<Node>,
    holes: Vec<(usize, usize)>,
    data: marker::PhantomData<[T]>,
}

impl Octree<Voxel> for SparseOctree<Voxel> {
    fn new() -> Self {
        let size = 0;
        let mut node_count = vec![];
        let mut nodes = vec![];
        let mut holes = vec![];

        node_count.push(0..=0);
        nodes.push(Node {
            morton: 0,
            child: u32::MAX,
            valid: 0,
            voxel: Voxel::air(),
        });

        Self {
            size,
            node_count,
            nodes,
            holes,
            data: marker::PhantomData,
        }
    }

    fn place(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        self.size = 9;

        let mut hierarchy = self.get_position_hierarchy(x, y, z);

        //dbg!(&hierarchy);

        let node = self.push_back(&hierarchy[..]);

        node.unwrap().voxel = voxel;

        //self.print_all();
    }
}

impl SparseOctree<Voxel> {
    pub fn nodes(&self) -> &'_ [Node] {
        &self.nodes[..]
    }

    pub fn get_node<'a>(&'a self, hierarchy: &[u8]) -> Option<(&'a Node, usize)> {
        let mut index = 0;

        for (level, &mask) in hierarchy.iter().enumerate() {
            if mask.count_ones() != 1 {
                panic!("invalid mask");
            }

            //dbg!(index);
            //dbg!("traversing node", self.nodes[index]);

            //println!("valid: {:#010b}", self.nodes[index].valid);
            //println!("mask : {:#010b}", mask);

            let p = (self.nodes[index].valid & (mask as u32 - 1)).count_ones();
            //dbg!(p);
            if self.nodes[index].valid & mask as u32 == mask as u32
                && self.nodes[index].child != u32::MAX
            {
                index = self.nodes[index].child as usize + p as usize;
            } else {
                return None;
            }
        }

        Some((&self.nodes[index], index))
    }

    fn push_back<'a>(&'a mut self, hierarchy: &[u8]) -> Option<&'a mut Node> {
        //println!("ADD NODE");

        let mut index = 0;

        for (level, &mask) in hierarchy.iter().enumerate() {
            if mask.count_ones() != 1 {
                panic!("invalid mask");
            }

            //dbg!(index);
            //dbg!("traversing node", self.nodes[index]);

            //println!("valid: {:#010b}", self.nodes[index].valid);
            //println!("mask : {:#010b}", mask);

            let p = (self.nodes[index].valid & (mask as u32 - 1)).count_ones();
            //dbg!(p);
            if self.nodes[index].valid & mask as u32 == mask as u32
                && self.nodes[index].child != u32::MAX
            {
                index = self.nodes[index].child as usize + p as usize;
            } else {
                let node = self.nodes[index];

                self.nodes[index].valid |= mask as u32;

                let p = (self.nodes[index].valid & (mask as u32 - 1)).count_ones();
                let q = self.nodes[index].valid.count_ones() - 1;

                self.nodes[index].child = self.nodes.len() as _;

                for i in 0..q {
                    let x = self.nodes[index].child as usize + i as usize;
                    let y = node.child as usize + i as usize;
                    let n = self.nodes[y];
                    self.nodes[y] = Node::default();
                    self.nodes.insert(x, n);
                }

                let child = self.nodes[index].child as usize + p as usize;

                self.nodes.insert(
                    child as _,
                    Node {
                        voxel: self.nodes[index].voxel,
                        ..Node::default()
                    },
                );
                self.nodes[child].morton = Self::get_morton_code(&hierarchy[..=level]);

                index = child as _;
            }
        }

        Some(&mut self.nodes[index])
    }

    pub fn get_morton_code(hierarchy: &[u8]) -> u64 {
        let mut morton = 0x7;

        for &mask in hierarchy {
            morton <<= 3;

            let index = mask.trailing_zeros() as u64;

            morton |= index & 0x7;
        }

        morton
    }

    pub fn optimize(&mut self) {
        let mut nodes = self.nodes.clone();

        for i in (0..nodes.len()).rev() {
            if nodes[i].child == u32::MAX {
                continue;
            }

            if nodes[i].valid != u8::MAX as _ {
                continue;
            }

            let child = nodes[i].child as usize;

            let children = child..child + 8;

            let first_child = nodes[child];

            if first_child.voxel.is_translucent() {
                continue;
            }

            let all_children_same = nodes[children.clone()]
                .iter()
                .all(|node| node.voxel.id == first_child.voxel.id);

            if all_children_same {
                nodes[i] = Node {
                    morton: nodes[i].morton,
                    child: u32::MAX,
                    valid: 0,
                    voxel: first_child.voxel,
                };
                //dbg!(nodes[i]);
                for j in children.clone() {
                    nodes[j] = Node::default();
                }
            }
        }

        nodes.retain(|node| node.morton != u64::MAX);

        nodes.sort_by(|a, b| a.morton.cmp(&b.morton));

        for i in 0..nodes.len() {
            if nodes[i].child == u32::MAX {
                continue;
            }

            let child: Node = self.nodes[nodes[i].child as usize];

            nodes[i].child = nodes
                .binary_search_by(|probe| probe.morton.cmp(&child.morton))
                .expect("failed to find node") as _;
        }

        self.nodes = nodes;
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn get_position_hierarchy(&self, mut x: usize, mut y: usize, mut z: usize) -> Vec<u8> {
        let mut hierarchy = vec![];

        let mut hsize = 2usize.pow(self.size as _);

        for i in 0..self.size {
            hsize /= 2;

            let px = (x >= hsize) as u8;
            let py = (y >= hsize) as u8;
            let pz = (z >= hsize) as u8;

            let index = px * 4 + py * 2 + pz;

            let mask = 1 << index;

            x -= px as usize * hsize;
            y -= py as usize * hsize;
            z -= pz as usize * hsize;

            hierarchy.push(mask);
        }

        hierarchy
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Node {
    child: u32,
    valid: u32,
    morton: u64,
    voxel: Voxel,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            morton: u64::MAX,
            child: u32::MAX,
            valid: 0,
            ..Self::default()
        }
    }
}
