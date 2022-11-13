use std::sync::Arc;

use vulkano::buffer::CpuAccessibleBuffer;
use bytemuck::Pod;

pub trait Update {
    fn update(& mut self) -> ();
}

pub trait UniformBufferOwner<T: Pod + Send + Sync> {
    fn get_uniform_buffer(&self) -> Arc<CpuAccessibleBuffer<T>>;
}