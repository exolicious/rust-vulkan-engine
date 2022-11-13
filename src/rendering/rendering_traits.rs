use cgmath::Vector3;

use super::primitives::{Triangle, Vertex};

pub trait Mesh {
    fn generate_mesh(bounds: Vector3<f32>) -> Vec<Triangle>;
    fn unwrap_vertices(&self) -> Vec<Vertex>;
    fn mesh_helper(&self) {}
}
