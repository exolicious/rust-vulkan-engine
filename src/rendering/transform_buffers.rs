use std::error::Error;
use std::sync::Arc;

use glam::Mat4;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};

const TRANSFORM_BUFFER_CAPACITY: usize = 4096;

/// One storage buffer per swapchain image, plus the CPU-side authoritative
/// model matrices indexed by entity. Each frame the matrices are written into
/// the buffer of the image being rendered, so the per-image buffers never need
/// to be synchronized with each other.
pub struct TransformBuffers {
    memory_allocator: Arc<StandardMemoryAllocator>,
    buffers: Vec<Subbuffer<[[[f32; 4]; 4]]>>,
    model_matrices: Vec<Mat4>,
}

impl TransformBuffers {
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>, swapchain_image_count: usize) -> Self {
        let buffers = Self::build_buffers(&memory_allocator, swapchain_image_count);
        Self {
            memory_allocator,
            buffers,
            model_matrices: Vec::new(),
        }
    }

    fn build_buffers(
        memory_allocator: &Arc<StandardMemoryAllocator>,
        count: usize,
    ) -> Vec<Subbuffer<[[[f32; 4]; 4]]>> {
        (0..count)
            .map(|_| {
                Buffer::from_iter(
                    memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::STORAGE_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_HOST
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    vec![[[0f32; 4]; 4]; TRANSFORM_BUFFER_CAPACITY],
                )
                .unwrap()
            })
            .collect()
    }

    pub fn set_image_count(&mut self, count: usize) {
        if count != self.buffers.len() {
            self.buffers = Self::build_buffers(&self.memory_allocator, count);
        }
    }

    pub fn push(&mut self, model_matrix: Mat4) -> Result<(), Box<dyn Error>> {
        if self.model_matrices.len() >= TRANSFORM_BUFFER_CAPACITY {
            return Err(format!(
                "transform buffer capacity ({TRANSFORM_BUFFER_CAPACITY}) exceeded"
            )
            .into());
        }
        self.model_matrices.push(model_matrix);
        Ok(())
    }

    pub fn set(&mut self, entity_index: usize, model_matrix: Mat4) {
        self.model_matrices[entity_index] = model_matrix;
    }

    /// Writes the model matrices into the given image's buffer, laid out in
    /// `order` (instance slots grouped by mesh, see `MeshAccessor::instance_order`).
    pub fn upload(
        &self,
        image_index: usize,
        order: impl Iterator<Item = usize>,
    ) -> Result<(), Box<dyn Error>> {
        let mut write_lock = self.buffers[image_index].write()?;
        for (slot, entity_index) in order.enumerate() {
            write_lock[slot] = self.model_matrices[entity_index].to_cols_array_2d();
        }
        Ok(())
    }

    pub fn buffer(&self, image_index: usize) -> Subbuffer<[[[f32; 4]; 4]]> {
        self.buffers[image_index].clone()
    }
}
