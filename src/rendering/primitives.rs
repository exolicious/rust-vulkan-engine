use glam::Vec3;
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
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
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
    [0, 2, 1],
    [2, 0, 3],
    [5, 7, 4],
    [7, 5, 6],
    [4, 3, 0],
    [3, 4, 7],
    [1, 6, 5],
    [6, 1, 2],
    [4, 1, 5],
    [1, 4, 0],
    [3, 6, 2],
    [6, 3, 7],
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
    fn tick(&mut self, frame_time: f32) -> Option<TickAction> {
        //return None;
        //let amount: f32 = rand::thread_rng().gen_range(-0.02..0.02);
        //self.transform.translation.x += amount;
        self.transform.rotation = self.transform.rotation * glam::Quat::from_rotation_y(-0.01 / frame_time);
        self.transform.rotation = self.transform.rotation * glam::Quat::from_rotation_x(-0.01 / frame_time);
        Some(TickAction::HasMoved(self.transform))
    }

    fn transform(&self) -> Transform {
        self.transform
    }

    fn mesh(&self) -> Mesh {
        let data = CUBE_TRIANGLES
            .iter()
            .flat_map(|triangle: &[usize; 3]| {
                let normal = calculate_normal_from_triangle(triangle);
                triangle.iter().map(move |&corner| {
                    let [x, y, z] = CUBE_CORNERS[corner];
                    Vertex {
                        position: [x, y, z],
                        color: [1.0, 1.0, 1.0],
                        normal: normal,
                    }
                })
            })
            .collect();
        Mesh::new("cube", data)
    }
}

fn calculate_normal_from_triangle(triangle: &[usize; 3]) -> [f32; 3] {
    let [a, b, c] = triangle.map(|corner| Vec3::from(CUBE_CORNERS[corner]));
    // Winding is counter-clockwise seen from outside, so (b - a) x (c - a) points outward.
    (b - a).cross(c - a).normalize_or_zero().to_array()
}