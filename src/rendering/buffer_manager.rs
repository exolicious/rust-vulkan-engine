use std::error::Error;
use std::sync::Arc;

use glam::Mat4;
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::descriptor_set::allocator::{
    StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo,
};
use vulkano::device::Device;
use vulkano::memory::allocator::StandardMemoryAllocator;

use crate::physics::physics_traits::Transform;

use super::frame::FrameInFlight;
use super::primitives::Mesh;
use super::transforms::Transforms;
use super::vertex_buffers::VertexBuffer;

pub struct BufferManager {
    pub descriptor_set_allocator: StandardDescriptorSetAllocator,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub vertex_buffer: VertexBuffer,
    transforms: Transforms,
}

impl BufferManager {
    pub fn new(device: Arc<Device>) -> Self {
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(
            device.clone(),
            StandardDescriptorSetAllocatorCreateInfo::default(),
        );
        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo {
                secondary_buffer_count: 32,
                ..Default::default()
            },
        );
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device));
        let vertex_buffer = VertexBuffer::new(memory_allocator.clone());

        Self {
            descriptor_set_allocator,
            command_buffer_allocator,
            memory_allocator,
            vertex_buffer,
            transforms: Transforms::new(),
        }
    }

    pub fn register_entity(
        &mut self,
        transform: Transform,
        mesh: Mesh,
        entity_index: usize,
    ) -> Result<(), Box<dyn Error>> {
        // Both capacity checks happen before either structure is mutated, so a
        // failed registration cannot leave the mesh groups and the transforms
        // disagreeing about which entities exist.
        self.transforms.check_capacity()?;
        self.vertex_buffer.register_mesh_instance(mesh, entity_index)?;
        self.transforms.push(transform.model_matrix());
        Ok(())
    }

    pub fn set_entity_transform(&mut self, entity_index: usize, transform: &Transform) {
        self.transforms.set(entity_index, transform.model_matrix());
    }

    pub fn has_pending_vertex_uploads(&self) -> bool {
        self.vertex_buffer.has_pending_uploads()
    }

    pub fn upload_frame_data(
        &mut self,
        frame: &FrameInFlight,
        view_projection: Mat4,
    ) -> Result<(), Box<dyn Error>> {
        self.vertex_buffer.flush_pending_uploads()?;
        self.transforms.write_into(
            &frame.transform_buffer,
            self.vertex_buffer.mesh_accessor.instance_order(),
        )?;
        *frame.vp_buffer.write()? = view_projection.to_cols_array_2d();
        Ok(())
    }
}
