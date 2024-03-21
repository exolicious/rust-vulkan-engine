use std::{ops::Index, sync::Arc};

use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}};

use super::primitives::Vertex;


pub struct TransformBuffers {
    transform_buffers: Vec<Subbuffer<[[[f32; 4]; 4]]>>
}
const INITIAL_TRANSFORM_BUFFER_SIZE: usize = 2_i32.pow(12) as usize; // 32 instances

impl TransformBuffers {
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>, swapchain_images_length: usize) -> Self {
        let mut transform_buffers = Vec::new();
        for _ in 0..swapchain_images_length {
            let transform_initial_data: [[[f32; 4]; 4]; INITIAL_TRANSFORM_BUFFER_SIZE] = [[[0_f32; 4]; 4]; INITIAL_TRANSFORM_BUFFER_SIZE];
            let uniform_buffer = Buffer::from_iter(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                transform_initial_data.into_iter()
            )
            .unwrap();
            transform_buffers.push(uniform_buffer);
        }

        Self {
            transform_buffers
        }
    }
}

impl Index<usize> for TransformBuffers {
    type Output = Subbuffer<[[[f32; 4]; 4]]>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.transform_buffers[index]
    }
}