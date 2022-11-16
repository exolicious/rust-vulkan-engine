use bytemuck::Pod;
use cgmath::Vector3;
use std::{sync::Arc, cell::RefCell, rc::Rc};
use vulkano::buffer::CpuAccessibleBuffer;
use crate::engine::general_traits::Entity;

use super::primitives::{Triangle, Vertex};

pub trait UniformBufferOwner<T: Pod + Send + Sync> {
    fn get_uniform_buffers(& self) -> Vec<Arc<CpuAccessibleBuffer<T>>>;
}

pub trait HasMesh {
    fn generate_mesh(&mut self) -> ();
    fn unwrap_vertices(&self) -> Vec<Vertex>;
    fn mesh_helper(&self) {}
}

pub trait UpdateGraphics : Entity {
    fn update_graphics(& self, swapchain_image_index: usize) -> ();
}

pub trait ModelBlueprint : HasMesh {}

pub trait HasEntities {
    fn push_entity(&mut self, entity: Rc<RefCell<dyn UpdateGraphics>>);

}
