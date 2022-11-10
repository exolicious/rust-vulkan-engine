use bytemuck::{Zeroable, Pod};

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position);

pub struct Triangle {
    pub vertices: [Vertex; 3]
}

impl Triangle {
    pub fn new(v1: Vertex, v2: Vertex, v3: Vertex) -> Self {
        Self {
            vertices: [v1, v2, v3]
        }
    }

    pub fn vertices(&self) -> Vec<f32> {
        let mut res = vec![0_f32;6];
        for vertex in self.vertices {
            res.append(& mut vertex.position.to_vec());
        }
        res
    }
}

