use std::alloc;
use std::fs;
use std::io::{self, BufRead};
use std::mem;
use std::ptr;
use std::slice;

#[derive(Clone, Copy)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uvw: [f32; 3],
}

type Index = u16;

pub struct Mesh {
    vertex_count: usize,
    index_count: usize,
    data: ptr::NonNull<u8>,
}

impl Mesh {
    pub fn from_obj(file: fs::File) -> Self {
        let buf_reader = io::BufReader::new(file);

        let mut vertices = vec![];
        let mut positions = vec![];
        let mut normals = vec![];
        let mut uvws = vec![];
        let mut indices = vec![];

        //not a totally accurate obj reader. made to read a single cube
        for line in buf_reader.lines() {
            let line = line.expect("failed to read line");
            let segments = line.split_whitespace().collect::<Vec<_>>();

            if segments.len() == 0 {
                continue;
            }

            match segments[0] {
                "v" => {
                    let position = [
                        segments[1].parse::<f32>().expect("failed to parse float"),
                        segments[2].parse::<f32>().expect("failed to parse float"),
                        segments[3].parse::<f32>().expect("failed to parse float"),
                    ];

                    positions.push(position);
                }
                "vt" => {
                    let uvw = [
                        segments[1].parse::<f32>().expect("failed to parse float"),
                        segments[2].parse::<f32>().expect("failed to parse float"),
                        segments[3].parse::<f32>().expect("failed to parse float"),
                    ];

                    uvws.push(uvw);
                }
                "vn" => {
                    let normal = [
                        segments[1].parse::<f32>().expect("failed to parse float"),
                        segments[2].parse::<f32>().expect("failed to parse float"),
                        segments[3].parse::<f32>().expect("failed to parse float"),
                    ];

                    normals.push(normal);
                }
                "f" => {
                    let parse_index = |id: &str| {
                        let y = id.split("/").collect::<Vec<_>>();

                        let i = y[0].parse::<usize>().unwrap_or_default();
                        let j = y[1].parse::<usize>().unwrap_or_default();
                        let k = y[2].parse::<usize>().unwrap_or_default();

                        //obj indices start at 1
                        (i - 1, j - 1, k - 1)
                    };

                    let (i1, j1, k1) = parse_index(segments[1]);
                    let (i2, j2, k2) = parse_index(segments[2]);
                    let (i3, j3, k3) = parse_index(segments[3]);

                    let vertex_1 = Vertex {
                        position: positions[i1],
                        uvw: uvws[j1],
                        normal: normals[k1],
                    };

                    let vertex_2 = Vertex {
                        position: positions[i2],
                        uvw: uvws[j2],
                        normal: normals[k2],
                    };

                    let vertex_3 = Vertex {
                        position: positions[i3],
                        uvw: uvws[j3],
                        normal: normals[k3],
                    };

                    vertices.push(vertex_1);
                    vertices.push(vertex_2);
                    vertices.push(vertex_3);

                    let index_count = indices.len();

                    indices.push(index_count as Index + 0);
                    indices.push(index_count as Index + 1);
                    indices.push(index_count as Index + 2);
                }
                _ => {}
            }
        }

        Mesh::create(&vertices, &indices)
    }

    pub fn create(vertices: &'_ [Vertex], indices: &'_ [Index]) -> Self {
        let vertex_byte_len = vertices.len() * mem::size_of::<Vertex>();
        let index_byte_len = indices.len() * mem::size_of::<Index>();
        let byte_len = vertex_byte_len + index_byte_len;

        let layout = alloc::Layout::array::<u8>(byte_len).expect("failed to create layout");

        let data = match ptr::NonNull::new(unsafe { alloc::alloc(layout) }) {
            Some(p) => p,
            None => alloc::handle_alloc_error(layout),
        };

        let (data_vertex, data_index) = unsafe {
            (
                data.as_ptr().cast::<_>(),
                data.as_ptr().add(vertex_byte_len).cast::<_>(),
            )
        };

        unsafe { ptr::copy(vertices.as_ptr(), data_vertex, vertices.len()) };
        unsafe { ptr::copy(indices.as_ptr(), data_index, indices.len()) };

        Self {
            vertex_count: vertices.len(),
            index_count: indices.len(),
            data,
        }
    }

    pub fn get(&self) -> (&'_ [Vertex], &'_ [Index]) {
        let vertices = unsafe {
            slice::from_raw_parts(
                self.data
                    .as_ptr()
                    .add(self.get_vertex_offset())
                    .cast::<Vertex>(),
                self.vertex_count,
            )
        };

        let indices = unsafe {
            slice::from_raw_parts(
                self.data
                    .as_ptr()
                    .add(self.get_index_offset())
                    .cast::<Index>(),
                self.index_count,
            )
        };

        (vertices, indices)
    }

    #[inline]
    fn get_vertex_offset(&self) -> usize {
        0
    }

    #[inline]
    fn get_index_offset(&self) -> usize {
        self.get_vertex_offset() + self.vertex_count * mem::size_of::<Vertex>()
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {}
}
