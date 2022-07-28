use math::prelude::Vector;

use std::ops::RangeInclusive;

pub const PAGE_SIZE: usize = 4000;

pub struct Octree {
    size: usize,
    node_count: Vec<RangeInclusive<usize>>,
    data: Vec<Node>,
    holes: Vec<(usize, usize)>,
}

impl Octree {
    pub fn new() -> Self {
        let size = 0;
        let mut node_count = vec![];
        let mut data = vec![];
        let mut holes = vec![];

        node_count.push(0..=0);
        data.push(Node {
            morton: 0,
            child: u32::MAX,
            valid: 0,
            block: 42069,
        });

        Octree {
            size,
            node_count,
            data,
            holes,
        }
    }

    pub fn place(&mut self, x: usize, y: usize, z: usize, cubelet: u16) {
        self.size = 5;

        let mut hierarchy = self.get_position_hierarchy(x, y, z);

        //dbg!(&hierarchy);

        let node = self.add_node(&hierarchy[..]);

        node.unwrap().block = cubelet as u32;

        //self.print_all();
    }

    pub fn data(&self) -> &'_ [Node] {
        &self.data[..]
    }

    pub fn get_node<'a>(&'a self, hierarchy: &[u8]) -> Option<(&'a Node, usize)> {
        let mut index = 0;

        for (level, &mask) in hierarchy.iter().enumerate() {
            if mask.count_ones() != 1 {
                panic!("invalid mask");
            }

            //dbg!(index);
            //dbg!("traversing node", self.data[index]);

            //println!("valid: {:#010b}", self.data[index].valid);
            //println!("mask : {:#010b}", mask);

            let p = (self.data[index].valid & (mask as u32 - 1)).count_ones();
            //dbg!(p);
            if self.data[index].valid & mask as u32 == mask as u32
                && self.data[index].child != u32::MAX
            {
                index = self.data[index].child as usize + p as usize;
            } else {
                return None;
            }
        }

        Some((&self.data[index], index))
    }

    fn add_node<'a>(&'a mut self, hierarchy: &[u8]) -> Option<&'a mut Node> {
        //println!("ADD NODE");

        let mut index = 0;

        for (level, &mask) in hierarchy.iter().enumerate() {
            if mask.count_ones() != 1 {
                panic!("invalid mask");
            }

            //dbg!(index);
            //dbg!("traversing node", self.data[index]);

            //println!("valid: {:#010b}", self.data[index].valid);
            //println!("mask : {:#010b}", mask);

            let p = (self.data[index].valid & (mask as u32 - 1)).count_ones();
            //dbg!(p);
            if self.data[index].valid & mask as u32 == mask as u32
                && self.data[index].child != u32::MAX
            {
                index = self.data[index].child as usize + p as usize;
            } else {
                let node = self.data[index];

                self.data[index].valid |= mask as u32;

                let p = (self.data[index].valid & (mask as u32 - 1)).count_ones();
                let q = self.data[index].valid.count_ones() - 1;

                self.data[index].child = self.data.len() as _;

                for i in 0..q {
                    let x = self.data[index].child as usize + i as usize;
                    let y = node.child as usize + i as usize;
                    let n = self.data[y];
                    self.data[y] = Node::default();
                    self.data.insert(x, n);
                }

                let child = self.data[index].child as usize + p as usize;

                self.data.insert(child as _, Node::default());
                self.data[child].morton = Self::get_morton_code(&hierarchy[..=level]);

                index = child as _;
            }
        }

        Some(&mut self.data[index])
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
        let mut data = self.data.clone();

        data.retain(|node| node.morton != u64::MAX);

        data.sort_by(|a, b| a.morton.cmp(&b.morton));

        for i in 0..data.len() {
            if data[i].child == u32::MAX {
                continue;
            }

            let child: Node = self.data[data[i].child as usize];

            data[i].child = data
                .binary_search_by(|probe| probe.morton.cmp(&child.morton))
                .expect("failed to find node") as _;
        }

        self.data = data;
    }

    pub fn print_all(&self) {
        dbg!(&self.node_count);
        dbg!(self.size);
        dbg!(self.data.len());
        let mut index = 0;
        let mut level = 0;
        let mut children = 0;
        for (i, node) in self.data.iter().enumerate() {
            children += node.valid.count_ones() as usize;
            index += 1;
            if (node.block != 42069 && node.block != 1 && node.block != 2 && node.block != 3) {
                println!("index {}", i);
                dbg!(node);
            }
        }
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
    block: u32,
    morton: u64,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            morton: u64::MAX,
            child: u32::MAX,
            valid: 0,
            block: 42069,
        }
    }
}
