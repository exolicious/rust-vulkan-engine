use std::sync::Arc;

use glam::Mat4;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::CommandBufferExecFuture;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::image::view::ImageView;
use vulkano::image::Image;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::{PresentFuture, SwapchainAcquireFuture};
use vulkano::sync::future::{FenceSignalFuture, JoinFuture};
use vulkano::sync::GpuFuture;

use super::transforms::TRANSFORM_BUFFER_CAPACITY;

/// How many frames the CPU may record ahead of the GPU. Independent of the
/// swapchain image count, which the presentation engine dictates.
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub type FrameEndFuture = FenceSignalFuture<
    PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>,
>;

/// Per-swapchain-image resources: only what is tied to the presented image.
pub struct Frame {
    pub swapchain_image_view: Arc<ImageView>,
    pub framebuffer: Arc<Framebuffer>,
}

impl Frame {
    pub fn new(swapchain_image: Arc<Image>, render_pass: Arc<RenderPass>) -> Self {
        let swapchain_image_view = ImageView::new_default(swapchain_image).unwrap();
        let framebuffer = Framebuffer::new(
            render_pass,
            FramebufferCreateInfo {
                attachments: vec![swapchain_image_view.clone()],
                ..Default::default()
            },
        )
        .unwrap();
        Self {
            swapchain_image_view,
            framebuffer,
        }
    }
}

/// Everything the CPU writes while the GPU may still be reading the previous
/// frame's copy. One instance per frame in flight; `fence` guards reuse of the
/// whole bundle.
pub struct FrameInFlight {
    pub vp_buffer: Subbuffer<[[f32; 4]; 4]>,
    pub transform_buffer: Subbuffer<[[[f32; 4]; 4]]>,
    pub vp_descriptor_set: Arc<PersistentDescriptorSet>,
    pub transform_descriptor_set: Arc<PersistentDescriptorSet>,
    pub fence: Option<Arc<FrameEndFuture>>,
}

impl FrameInFlight {
    pub fn new(
        memory_allocator: Arc<StandardMemoryAllocator>,
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
        vp_layout: Arc<DescriptorSetLayout>,
        transform_layout: Arc<DescriptorSetLayout>,
    ) -> Self {
        let vp_buffer = Buffer::from_data(
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
        .unwrap();

        let transform_buffer = Buffer::from_iter(
            memory_allocator,
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
        .unwrap();

        let vp_descriptor_set = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            vp_layout,
            [WriteDescriptorSet::buffer(0, vp_buffer.clone())],
            [],
        )
        .unwrap();
        let transform_descriptor_set = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            transform_layout,
            [WriteDescriptorSet::buffer(0, transform_buffer.clone())],
            [],
        )
        .unwrap();

        Self {
            vp_buffer,
            transform_buffer,
            vp_descriptor_set,
            transform_descriptor_set,
            fence: None,
        }
    }
}
