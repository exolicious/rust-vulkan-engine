use bytemuck::Pod;
use std::{sync::Arc};
use vulkano::buffer::CpuAccessibleBuffer;
use crate::engine::general_traits::Entity;

use super::primitives::{Vertex};

pub trait UniformBufferOwner<T: Pod + Send + Sync> {
    fn get_uniform_buffers(& self) -> Vec<Arc<CpuAccessibleBuffer<T>>>;
}

pub trait HasMesh : Entity  {
    fn generate_mesh(&mut self) -> ();
    fn unwrap_vertices(&self, ) -> Vec<Vertex>;
    fn set_hash(&mut self, hash: u64) -> ();
    fn get_hash(&self) -> u64;
    fn mesh_hash(&mut self) -> ();
}

pub trait UpdateGraphics : Entity {
    fn update_graphics(& self, swapchain_image_index: usize) -> ();
}


pub trait RenderableEntity : Entity + UpdateGraphics + HasMesh {}

pub type MatrixBufferData = [[f32;4];4];