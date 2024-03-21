use std::{ops::Index, sync::Arc};

use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}};

use super::primitives::Vertex;

pub struct VertexBuffers {
    vertex_buffers: Vec<Subbuffer<[Vertex]>>,
    entity_id_to_tranform_id: Vec<usize>
}
const INITIAL_VERTEX_BUFFER_SIZE: usize = 2_i32.pow(16) as usize; 

impl VertexBuffers {
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>, swapchain_images_length: usize) -> Self {
        let mut vertex_buffers = Vec::new();
        for _ in 0..swapchain_images_length {
            let initializer_data = vec![Vertex{position: [0.,0.,0.]}; INITIAL_VERTEX_BUFFER_SIZE];
            let vertex_buffer = Buffer::from_iter(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                initializer_data.into_iter()
            )
            .unwrap();
            vertex_buffers.push(vertex_buffer);
        }
        let entity_id_to_tranform_id = vec![INITIAL_VERTEX_BUFFER_SIZE];
        Self {
            vertex_buffers,
            entity_id_to_tranform_id
        }
    }
}

impl Index<usize> for VertexBuffers {
    type Output = Subbuffer<[Vertex]>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vertex_buffers[index]
    }
}