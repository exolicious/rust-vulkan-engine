use bytemuck::Pod;
use cgmath::Vector3;
use std::sync::Arc;
use vulkano::buffer::CpuAccessibleBuffer;
use super::primitives::{Triangle, Vertex};

pub trait UniformBufferOwner<T: Pod + Send + Sync> {
    fn get_uniform_buffers(& self) -> Vec<Arc<CpuAccessibleBuffer<T>>>;
}

pub trait Mesh {
    fn generate_mesh(bounds: Vector3<f32>) -> Vec<Triangle>;
    fn unwrap_vertices(&self) -> Vec<Vertex>;
    fn mesh_helper(&self) {}
}

pub trait UpdateGraphics {
    fn update_graphics(& self, swapchain_image_index: usize) -> ();
}
