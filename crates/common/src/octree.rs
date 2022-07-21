use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::cmp;
use std::ops::RangeInclusive;
use std::ptr;
use std::slice;

pub const PAGE_SIZE: usize = 4000;

pub struct Octree {
    size: usize,
    node_count: Vec<RangeInclusive<usize>>,
    data: Vec<Node>,
}

impl Octree {
    pub fn new() -> Self {
        let size = 0;
        let mut node_count = vec![];
        let mut data = Vec::with_capacity(1000000000);

        node_count.push(0..=0);
        data.push(Node::default());

        Octree {
            size,
            node_count,
            data,
        }
    }

    pub fn place(&mut self, x: usize, y: usize, z: usize, cubelet: u16) {
        self.size = 6;

        let mut hierarchy = self.get_position_hierarchy(x, y, z);

        //dbg!(&hierarchy);

        let node = self.add_node(&hierarchy[..]);

        node.unwrap().blocks = cubelet as u32;

        //self.print_all();
    }

    pub fn data(&self) -> &'_ [Node] {
        &self.data[..]
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
                self.data[index].valid |= mask as u32;

                let p = (self.data[index].valid & (mask as u32 - 1)).count_ones();
                //dbg!(p);
                if self.data[index].child == u32::MAX {
                    self.data[index].child = self.data.len() as _;
                }

                let child = self.data[index].child + p as u32;

                for i in 0..self.data.len() {
                    if self.data[i].child != u32::MAX {
                        if self.data[i].child >= child as _ && i != index {
                            self.data[i].child += 1;
                        }
                    }
                }

                self.data.insert(child as _, Node::default());

                index = child as _;
            }
        }

        Some(&mut self.data[index])
    }

    pub fn print_all(&self) {
        dbg!(&self.node_count);
        dbg!(self.size);
        dbg!(self.data.len());
        let mut index = 0;
        let mut level = 0;
        let mut children = 0;
        for (i, node) in self.data.iter().enumerate() {
            if index >= children {
                println!("level {}", level);
                level += 1;
                index = 0;
                children = 0;
            }

            children += node.valid.count_ones() as usize;
            index += 1;
            println!("index {}", i);
            dbg!(node);
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    fn get_position_hierarchy(&self, mut x: usize, mut y: usize, mut z: usize) -> Vec<u8> {
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
    blocks: u32,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            child: u32::MAX,
            valid: 0,
            blocks: 42069,
        }
    }
}
