use std::sync::Arc;

use vulkano::{image::{SwapchainImage, view::ImageView}, render_pass::{Framebuffer, RenderPass, FramebufferCreateInfo}, device::Device, 
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, CommandBufferUsage, RenderPassBeginInfo, SubpassContents}, 
    pipeline::{GraphicsPipeline, PipelineBindPoint, Pipeline}, buffer::{CpuAccessibleBuffer, TypedBufferAccess}, descriptor_set::PersistentDescriptorSet};
use winit::window::Window;

use super::primitives::Vertex;

pub struct Frame {
    swapchain_image: Arc<SwapchainImage<Window>>,
    render_pass: Arc<RenderPass>,
    device: Arc<Device>, 
    active_queue_family_index: u32, 
    pipeline: Arc<GraphicsPipeline>, 
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    uniform_buffer_descriptor_set: Arc<PersistentDescriptorSet>,
    transform_buffer_descriptor_set: Arc<PersistentDescriptorSet>,
    framebuffer: Option<Arc<Framebuffer>>,
    pub command_buffer: Option<Arc<PrimaryAutoCommandBuffer>>
}

impl Frame {
    pub fn new(swapchain_image: Arc<SwapchainImage<Window>>, render_pass: Arc<RenderPass>, device: Arc<Device>, active_queue_family_index: u32, pipeline: Arc<GraphicsPipeline>, 
        vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>, uniform_buffer_descriptor_set: Arc<PersistentDescriptorSet>, transform_buffer_descriptor_set: Arc<PersistentDescriptorSet>) -> Self {
        Self {
            swapchain_image,
            render_pass,
            device,
            active_queue_family_index,
            pipeline,
            vertex_buffer,
            uniform_buffer_descriptor_set,
            transform_buffer_descriptor_set,
            framebuffer: None,
            command_buffer: None,
        }
    }

    pub fn init(&mut self) {
        self.init_framebuffer();
        self.init_command_buffer();
    }

    fn init_framebuffer(&mut self) -> () {
        let view = ImageView::new_default(self.swapchain_image.clone()).unwrap();
        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            })
            .unwrap();
        self.framebuffer = Some(framebuffer);
    }

    pub fn init_command_buffer(&mut self) -> () {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.active_queue_family_index,
            CommandBufferUsage::MultipleSubmit,
        )
        .unwrap();

        command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(self.framebuffer.as_ref().unwrap().clone())
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                self.uniform_buffer_descriptor_set.clone(),
            )
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                self.transform_buffer_descriptor_set.clone(),
            )
            .draw(32, 1, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();
        
        let command_buffer = Arc::new(command_buffer_builder.build().unwrap());
        self.command_buffer = Some(command_buffer);
    }

    pub fn add_vertex_buffer(&mut self, vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>) {
        self.vertex_buffer = vertex_buffer;
        self.init_command_buffer();
    }
}