use rand::Rng;
use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex as VertexMacro;

use crate::engine::general_traits::{Entity, TickAction};
use crate::physics::Transform;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, BufferContents, VertexMacro)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub name: String,
    pub data: Vec<Vertex>,
}

impl Mesh {
    pub fn new(name: impl Into<String>, data: Vec<Vertex>) -> Self {
        Self {
            name: name.into(),
            data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cube {
    transform: Transform
}

// Corner indices: bit pattern x, y, z with 0 = negative and 1 = positive half-extent.
const CUBE_CORNERS: [[f32; 3]; 8] = [
    [-1., -1., -1.],
    [1., -1., -1.],
    [1., 1., -1.],
    [-1., 1., -1.],
    [-1., -1., 1.],
    [1., -1., 1.],
    [1., 1., 1.],
    [-1., 1., 1.],
];

const CUBE_TRIANGLES: [[usize; 3]; 12] = [
    [0, 1, 2],
    [2, 3, 0],
    [5, 4, 7],
    [7, 6, 5],
    [4, 0, 3],
    [3, 7, 4],
    [1, 5, 6],
    [6, 2, 1],
    [4, 5, 1],
    [1, 0, 4],
    [3, 2, 6],
    [6, 7, 3],
];

impl Cube {
    pub fn new(transform: Transform) -> Self {
        Self { transform }
    }
}

impl Default for Cube {
    fn default() -> Self {
        Self::new(Transform::default())
    }
}

impl Entity for Cube {
    fn tick(&mut self) -> Option<TickAction> {
        return None;
        let amount: f32 = rand::thread_rng().gen_range(-0.02..0.02);
        self.transform.translation.x += amount;
        Some(TickAction::HasMoved(self.transform))
    }

    fn transform(&self) -> Transform {
        self.transform
    }

    fn mesh(&self) -> Mesh {
        let data = CUBE_TRIANGLES
            .iter()
            .flat_map(|triangle| {
                triangle.iter().map(|&corner| {
                    let [x, y, z] = CUBE_CORNERS[corner];
                    Vertex {
                        position: [x, y, z],
                    }
                })
            })
            .collect();
        Mesh::new("cube", data)
    }
}
