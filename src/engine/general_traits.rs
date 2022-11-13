use std::sync::Arc;

use vulkano::buffer::CpuAccessibleBuffer;
use bytemuck::Pod;

pub trait Update {
    fn update(& mut self, swapchain_image_index: usize) -> ();
}

pub trait UniformBufferOwner<T: Pod + Send + Sync> {
    fn get_uniform_buffers(& self) -> Vec<Arc<CpuAccessibleBuffer<T>>>;
}