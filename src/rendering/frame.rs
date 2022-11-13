use std::sync::Arc;

use vulkano::{image::{SwapchainImage, view::ImageView}, render_pass::{Framebuffer, RenderPass, FramebufferCreateInfo}, device::Device, command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, CommandBufferUsage, RenderPassBeginInfo, SubpassContents}, pipeline::{GraphicsPipeline, PipelineBindPoint, Pipeline}, buffer::{CpuAccessibleBuffer, TypedBufferAccess}, descriptor_set::PersistentDescriptorSet};
use winit::window::Window;

use super::primitives::Vertex;

pub struct Frame {
/*     swapchain_image: Arc<SwapchainImage<Window>>,
    framebuffer: Arc<Framebuffer>, */
    pub command_buffer: Arc<PrimaryAutoCommandBuffer>
}

impl Frame {
    pub fn new(swapchain_image: Arc<SwapchainImage<Window>>, render_pass: Arc<RenderPass>, device: Arc<Device>, active_queue_family_index: u32, pipeline: Arc<GraphicsPipeline>, vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>, uniform_buffer_descriptor_set: Arc<PersistentDescriptorSet>) -> Self {
        let framebuffer = Self::create_framebuffer(swapchain_image.clone(), render_pass);
        let command_buffer = Self::create_command_buffer(framebuffer.clone(), device, active_queue_family_index, pipeline, vertex_buffer, uniform_buffer_descriptor_set);
        Self {
           /*  swapchain_image,
            framebuffer, */
            command_buffer
        }
    }

    pub fn create_framebuffer(swapchain_image: Arc<SwapchainImage<Window>>, render_pass: Arc<RenderPass>) -> Arc<Framebuffer> {
        let view = ImageView::new_default(swapchain_image).unwrap();
        Framebuffer::new(render_pass.clone(),
                        FramebufferCreateInfo {
                            attachments: vec![view],
                            ..Default::default()
                        })
                        .unwrap()
    }

    pub fn create_command_buffer(framebuffer: Arc<Framebuffer>, device: Arc<Device>, active_queue_family_index: u32, pipeline: Arc<GraphicsPipeline>, vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>, uniform_buffer_descriptor_set: Arc<PersistentDescriptorSet>) -> Arc<PrimaryAutoCommandBuffer> {
            let mut builder = AutoCommandBufferBuilder::primary(
                device,
                active_queue_family_index,
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    },
                    SubpassContents::Inline,
                )
                .unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline.layout().clone(),
                    0,
                    uniform_buffer_descriptor_set,
                )
                .draw(vertex_buffer.len() as u32, 1, 0, 0)
                .unwrap()
                .end_render_pass()
                .unwrap();
            Arc::new(builder.build().unwrap())
    }
}