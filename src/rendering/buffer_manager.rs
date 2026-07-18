use std::error::Error;
use std::sync::Arc;

use glam::Mat4;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::descriptor_set::allocator::{
    StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo,
};
use vulkano::device::Device;
use vulkano::image::Image;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::render_pass::RenderPass;

use crate::physics::physics_traits::Transform;

use super::frame::Frame;
use super::primitives::Mesh;
use super::transform_buffers::TransformBuffers;
use super::vertex_buffers::VertexBuffer;

pub struct BufferManager {
    pub descriptor_set_allocator: StandardDescriptorSetAllocator,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub vertex_buffer: VertexBuffer,
    pub transform_buffers: TransformBuffers,
    vp_buffers: Vec<Subbuffer<[[f32; 4]; 4]>>,
    pub frames: Vec<Frame>,
}

impl BufferManager {
    pub fn new(
        device: Arc<Device>,
        swapchain_images: Vec<Arc<Image>>,
        render_pass: Arc<RenderPass>,
    ) -> Self {
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

        let image_count = swapchain_images.len();
        let vertex_buffer = VertexBuffer::new(memory_allocator.clone());
        let transform_buffers = TransformBuffers::new(memory_allocator.clone(), image_count);
        let vp_buffers = Self::build_vp_buffers(&memory_allocator, image_count);
        let frames = Self::build_frames(swapchain_images, render_pass);

        Self {
            descriptor_set_allocator,
            command_buffer_allocator,
            memory_allocator,
            vertex_buffer,
            transform_buffers,
            vp_buffers,
            frames,
        }
    }

    fn build_frames(swapchain_images: Vec<Arc<Image>>, render_pass: Arc<RenderPass>) -> Vec<Frame> {
        swapchain_images
            .into_iter()
            .map(|image| Frame::new(image, render_pass.clone()))
            .collect()
    }

    fn build_vp_buffers(
        memory_allocator: &Arc<StandardMemoryAllocator>,
        count: usize,
    ) -> Vec<Subbuffer<[[f32; 4]; 4]>> {
        (0..count)
            .map(|_| {
                Buffer::from_data(
                    memory_allocator.clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::UNIFORM_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_HOST
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                    Mat4::IDENTITY.to_cols_array_2d(),
                )
                .unwrap()
            })
            .collect()
    }

    /// Called after the swapchain was recreated: rebuilds the framebuffers and,
    /// if the image count changed, the per-image buffers.
    pub fn rebuild_frames(
        &mut self,
        swapchain_images: Vec<Arc<Image>>,
        render_pass: Arc<RenderPass>,
    ) {
        let image_count = swapchain_images.len();
        if image_count != self.vp_buffers.len() {
            self.vp_buffers = Self::build_vp_buffers(&self.memory_allocator, image_count);
            self.transform_buffers.set_image_count(image_count);
        }
        self.frames = Self::build_frames(swapchain_images, render_pass);
    }

    pub fn register_entity(
        &mut self,
        transform: Transform,
        mesh: Mesh,
        entity_index: usize,
    ) -> Result<(), Box<dyn Error>> {
        self.vertex_buffer.register_mesh_instance(mesh, entity_index)?;
        self.transform_buffers.push(transform.model_matrix())
    }

    pub fn set_entity_transform(&mut self, entity_index: usize, transform: &Transform) {
        self.transform_buffers.set(entity_index, transform.model_matrix());
    }

    pub fn has_pending_vertex_uploads(&self) -> bool {
        self.vertex_buffer.has_pending_uploads()
    }

    pub fn upload_frame_data(
        &mut self,
        image_index: usize,
        view_projection: Mat4,
    ) -> Result<(), Box<dyn Error>> {
        self.vertex_buffer.flush_pending_uploads()?;
        self.transform_buffers
            .upload(image_index, self.vertex_buffer.mesh_accessor.instance_order())?;
        *self.vp_buffers[image_index].write()? = view_projection.to_cols_array_2d();
        Ok(())
    }

    pub fn vp_buffer(&self, image_index: usize) -> Subbuffer<[[f32; 4]; 4]> {
        self.vp_buffers[image_index].clone()
    }
}
