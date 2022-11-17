use bytemuck::Pod;
use std::{sync::Arc};
use vulkano::buffer::CpuAccessibleBuffer;
use crate::engine::general_traits::Entity;

use super::primitives::{Vertex};

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub trait UniformBufferOwner<T: Pod + Send + Sync> {
    fn get_uniform_buffers(& self) -> Vec<Arc<CpuAccessibleBuffer<T>>>;
}

pub trait HasMesh : Entity  {
    fn generate_mesh(&mut self) -> ();
    fn unwrap_vertices(&self) -> Vec<Vertex>;
/*     fn set_hash(&self) -> ();
    fn get_hash(&self) -> ();
    fn mesh_hash(&self) {
        let mut hasher = DefaultHasher::new();
        hasher.write(self.unwrap_vertices());
        self.set_hash();
    } */
}

pub trait UpdateGraphics : Entity {
    fn update_graphics(& self, swapchain_image_index: usize) -> ();
}


pub trait RenderableEntity : Entity + UpdateGraphics + HasMesh {}

pub type MatrixBufferData = [[f32;4];4];