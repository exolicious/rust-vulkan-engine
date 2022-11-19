use bytemuck::Pod;
use std::{sync::Arc};
use vulkano::buffer::CpuAccessibleBuffer;
use crate::engine::general_traits::Entity;

use super::primitives::{Triangle, Mesh};

pub trait UniformBufferOwner<T: Pod + Send + Sync> {
    fn get_uniform_buffers(& self) -> Vec<Arc<CpuAccessibleBuffer<T>>>;
}

pub trait HasMesh : Entity  {
    fn get_triangles(& self) -> Vec<Triangle>;
    fn generate_mesh(& self) -> Mesh;
}

pub trait UpdateGraphics : Entity {
    fn update_graphics(& self, swapchain_image_index: usize) -> ();
}


pub trait RenderableEntity : Entity + UpdateGraphics + HasMesh {}

pub type MatrixBufferData = [[f32;4];4];